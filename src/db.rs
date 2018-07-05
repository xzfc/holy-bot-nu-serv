use rusqlite::Connection;
use rusqlite::types::ToSql;
use super::db_util;
use super::error::MyError;
use super::process_log;
use super::serde_json;

use telegram_bot_raw::{Update, UpdateKind, User, Integer, MessageOrChannelPost, MessageChat};

use rand::{thread_rng, Rng};

pub struct Db {
    conn: Connection,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    start_day: i64,
    daily_users: Vec<i64>,
    daily_messages: Vec<i64>,

    messages_by_hour: [i64; 24],
    messages_by_weekday: [i64; 7],

    user_ids: Vec<String>,
    user_names: Vec<String>,
    messages_by_user: Vec<i64>,
}

impl Db {
    pub fn new(path: &str) -> Self {
        Db { conn: Connection::open(path).unwrap() }
    }

    pub fn update_from_file(self: &mut Self, path: &str) {
        let err = process_log::process_log(path, self);
        eprintln!("err = {:?}", err);
    }

    pub fn query(
        &self,
        chat: &str,
        dates: Option<(i64, i64)>,
        user_id: Option<&str>,
    ) -> (u16 ,String) {
        match self.query_inner(chat, dates, user_id) {
            Ok(res) => res,
            Err(e) => (500, format!("Error:\n{:?}", e)),
        }
    }

    /**************************************************************************/
    /*                           Private functions                            */
    /**************************************************************************/


    fn update_user(self: &mut Self, user: &User) -> Result<(), MyError> {
        let full_name = match &user.last_name {
            Some(last_name) => format!("{} {}", user.first_name, last_name),
            // XXX: actually, cloning here is redundant
            None => user.first_name.clone(),
        };

        let val = self.conn.execute("
            UPDATE users
               SET full_name = ?2
             WHERE user_id = ?1
            ", &[&Integer::from(user.id), &full_name])?;

        if val == 0 {
            self.conn.execute("
                INSERT INTO users VALUES
                ( ?1, ?2, ?3 )
                ", &[&Integer::from(user.id), &full_name, &random_id()])?;
        }
        Ok(())
    }

    fn update_chat(
        self: &mut Self,
        id: i64, 
        title: &String,
        username: &Option<String>,
        ) -> Result<(), MyError>
    {
        let username = username.clone().map(|x| format!("@{}", x));
        let val = self.conn.execute("
            UPDATE chats
               SET title = ?2
                 , username = ?3
             WHERE chat_id = ?1
            ", &[
                &id,
                title,
                &username,
            ])?;
        if val == 0 {
            self.conn.execute("
                INSERT INTO chats VALUES
                ( ?1, ?2, ?3, ?4 )
                ", &[
                    &id,
                    title,
                    &username,
                    &random_id(),
                ])?;
        }
        Ok(())
    }

    fn update(self: &mut Self, upd: Update) -> Result<(), MyError> {
        if let UpdateKind::Message(msg) = upd.kind {
            self.conn.execute("
                INSERT INTO messages VALUES
                ( ?1, ?2, ?3, ?4, 1 )
                ON CONFLICT (chat_id, user_id, day, hour)
                DO UPDATE SET count = count + 1
            ", &[
                &Integer::from(msg.chat.id()),
                &Integer::from(msg.from.id),
                &(msg.date/60/60/24),
                &(msg.date/60/60%24),
            ])?;

            self.update_user(&msg.from)?;

            match &msg.chat {
                MessageChat::Private(_) => return Ok(()),
                MessageChat::Unknown(_) => return Ok(()),
                MessageChat::Group(c)   =>
                    self.update_chat(
                        Integer::from(c.id),
                        &c.title,
                        &None,
                    )?,
                MessageChat::Supergroup(c) =>
                    self.update_chat(
                        Integer::from(c.id),
                        &c.title,
                        &c.username,
                    )?,
            }

            if let Some(reply) = msg.reply_to_message {
                if let MessageOrChannelPost::Message(reply) = *reply {
                    self.conn.execute("
                        INSERT INTO replies VALUES
                        ( ?1, ?2, ?3, 1 )
                        ON CONFLICT (chat_id, uid_from, uid_to)
                        DO UPDATE SET count = count + 1
                        ", &[
                            &Integer::from(msg.chat.id()),
                            &Integer::from(msg.from.id),
                            &Integer::from(reply.from.id),
                        ])?;

                    self.update_user(&reply.from)?;
                }
            }
        }

        Ok(())
    }

    pub fn query_inner(
        &self,
        chat: &str,
        dates: Option<(i64, i64)>,
        user_rid: Option<&str>,
    ) -> Result<(u16, String), MyError> {
        let chat_id = match self.search_chat(chat) {
            Some(chat_id) => chat_id,
            None => {
                return Ok((404, String::from(r#"{"error":"chat not found"}"#)));
            }
        };

        let mut result = QueryResult {
            start_day: 0,
            daily_users: Vec::new(),
            daily_messages: Vec::new(),
            messages_by_hour: [0; 24],
            messages_by_weekday: [0; 7],
            user_ids: Vec::new(),
            user_names: Vec::new(),
            messages_by_user: Vec::new(),
        };

        let mut _user_id: i64 = 0; // XXX
        let mut args: Vec<(&str, &ToSql)> = Vec::new();
        args.push((":chat_id", &chat_id));

        let mut filter = String::from("");
        if let Some(user_rid) = user_rid.as_ref() {
            let user_id =
                match self.search_user(user_rid) {
                    Some(user_id) => user_id,
                    None =>
                        return Ok((
                                404, String::from(
                                    r#"{"error":"user not found"}"#))),
                };

            filter += "AND :user_id = messages.user_id ";
            _user_id = user_id;
            args.push((":user_id",  &_user_id));
        }
        if let Some(dates) = dates.as_ref() {
            filter += "AND day BETWEEN :day_from AND :day_to ";
            args.push((":day_from", &dates.0));
            args.push((":day_to",   &dates.1));
        }
        let args = args.as_slice();

        let mut prev_day = 0;
        db_util::query_map_named(
            &self.conn,
            format!("
                SELECT day, COUNT(DISTINCT user_id), SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                       {}
                 GROUP BY day
            ", filter).as_ref(),
            args,
            |row| {
                let day = row.get(0);
                if result.start_day == 0 {
                    result.start_day = day;
                } else {
                    for _ in prev_day + 1..day {
                        result.daily_users.push(0);
                        result.daily_messages.push(0);
                    }
                }
                prev_day = day;
                result.daily_users.push(row.get(1));
                result.daily_messages.push(row.get(2));
            },
        )?;

        db_util::query_map_named(
            &self.conn,
            format!("
                SELECT hour, SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                       {}
                 GROUP BY hour
            ", filter).as_ref(),
            &args,
            |row| {
                let hour: i64 = row.get(0);
                result.messages_by_hour[hour as usize] = row.get(1);
            },
        )?;

        db_util::query_map_named(
            &self.conn,
            format!("
                SELECT (day+4)%7, SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                       {}
                 GROUP BY (day+4)%7
            ", filter).as_ref(),
            &args,
            |row| {
                let weekday: i64 = row.get(0);
                result.messages_by_weekday[weekday as usize] = row.get(1);
            },
        )?;

        db_util::query_map_named(
            &self.conn,
            format!("
                SELECT users.random_id,
                       users.full_name,
                       SUM(count)
                  FROM messages
                 INNER JOIN users ON users.user_id = messages.user_id
                 WHERE chat_id = :chat_id
                       {}
                 GROUP BY(messages.user_id)
                 ORDER BY SUM(COUNT) DESC
            ", filter).as_ref(),
            &args,
            |row| {
                result.user_ids.push(row.get(0));
                result.user_names.push(row.get(1));
                result.messages_by_user.push(row.get(2));
            },
        )?;

        return Ok((200, serde_json::to_string(&result).unwrap()))
    }

    fn search_chat(&self, chat: &str) -> Option<i64> {
        let res = self.conn.query_row(
            "
                SELECT chat_id
                  FROM chats
                 WHERE username = ?1
                    OR random_id = ?1
            ",
            &[&chat],
            |row| row.get::<_,i64>(0)
            );
        match res {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    fn search_user(&self, random_id: &str) -> Option<i64> {
        let res = self.conn.query_row(
            "
                SELECT user_id
                  FROM users
                 WHERE random_id = ?
            ",
            &[&random_id],
            |row| row.get::<_, i64>(0)
            );
        match res {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }
}

impl process_log::LogProcessor for Db {
    type Error = MyError;
    fn begin(&mut self) -> Result<Option<u64>, Self::Error> {
        self.conn.execute("BEGIN", &[])?;
        let seek = self.conn
            .query_row(
                "SELECT value FROM seek WHERE name = 'telegram'",
                &[],
                |row| row.get::<_,i64>(0));
        match seek {
            Ok(value) => Ok(Some(value as u64)),
            Err(_) => Ok(None),
        }
    }
    fn commit(&mut self, end_pos: u64) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO seek VALUES ('telegram', ?)",
            &[&(end_pos as i64)])?;
        self.conn.execute("COMMIT", &[])?;
        Ok(())
    }
    fn abort(&mut self) -> Result<(), Self::Error> {
        self.conn.execute("ROLLBACK", &[])?;
        Ok(())
    }
    fn process_line(&mut self, line: &String) -> Result<(), Self::Error> {
        match serde_json::from_str::<Update>(line) {
            Ok(upd) => { self.update(upd) },
            Err(err) => {
                eprintln!("Line: {}\nParse error: {}\n", line, err);
                Ok(())
            }
        }
    }
}

fn random_id() -> String {
    let mut result = String::from("");
    let mut rng = thread_rng();
    for _ in 0..8 {
        let v = (rng.gen::<u32>() % (10+26+26)) as u8;
        let v = v + match v {
            00..=09 => '0' as u8,
            10..=35 => 'a' as u8 - 10,
            36..=61 => 'A' as u8 - 36,
            _ => 0,
        };
        result.push(v as char);
    }
    result
}
