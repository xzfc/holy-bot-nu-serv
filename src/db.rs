use rusqlite::Connection;
use rusqlite::types::ToSql;
use super::db_util;
use super::serde_json;
use super::process_log;
use super::error::MyError;

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

    user_names: Vec<String>,
    messages_by_user: Vec<i64>,
}

impl Db {
    pub fn new(path: &str) -> Self {
        Db { conn: Connection::open(path).unwrap() }
    }

    pub fn init(self: &mut Self) {
        db_util::execute_many(&mut self.conn, "
            CREATE TABLE IF NOT EXISTS messages (
                chat_id INTEGER,
                user_id INTEGER,
                day     INTEGER,
                hour    INTEGER,
                count   INTEGER,
                PRIMARY KEY (chat_id, user_id, day, hour)
            );

            CREATE INDEX IF NOT EXISTS messages_i0
            ON messages ( chat_id, day );

            CREATE INDEX IF NOT EXISTS messages_i1
            ON messages ( chat_id, (day+4) % 7 );

            CREATE TABLE IF NOT EXISTS users (
                user_id   INTEGER,
                full_name TEXT,
                PRIMARY KEY (user_id)
            );

            CREATE TABLE IF NOT EXISTS replies (
                chat_id  INTEGER,
                uid_from INTEGER,
                uid_to   INTEGER,
                count    INTEGER,
                PRIMARY KEY (chat_id, uid_from, uid_to)
            );

            CREATE TABLE IF NOT EXISTS chats (
                chat_id   INTEGER,
                title     TEXT NOT NULL,
                username  TEXT,
                random_id TEXT NOT NULL,
                PRIMARY KEY (chat_id)
            );

            CREATE TABLE IF NOT EXISTS seek (
                name  TEXT,
                value INTEGER,
                PRIMARY KEY (name)
            );
            ");
    }

    pub fn update_from_file(self: &mut Self, path: &str) {
        let err = process_log::process_log(path, self);
        eprintln!("err = {:?}", err);
    }

    /**************************************************************************/
    /*                           Private functions                            */
    /**************************************************************************/


    fn update_user(self: &mut Self, user: &User) {
        let full_name = match &user.last_name {
            Some(last_name) => format!("{} {}", user.first_name, last_name),
            // XXX: actually, cloning here is redundant
            None => user.first_name.clone(),
        };
        self.conn.execute("
            INSERT OR REPLACE INTO users VALUES
            ( ?1, ?2 )
            ", &[&Integer::from(user.id), &full_name]
            ).unwrap();
    }

    fn update_chat(
        self: &mut Self,
        id: i64, 
        title: &String,
        username: &Option<String>,
        ) -> Result<(), MyError>
    {
        let val = self.conn.execute("
            UPDATE chats
               SET title = ?2
                 , username = ?3
             WHERE chat_id = ?1
            ", &[
                &id,
                title,
                username,
            ])?;
        if val == 0 {
            self.conn.execute("
                INSERT INTO chats VALUES
                ( ?1, ?2, ?3, ?4 )
                ", &[
                    &id,
                    title,
                    username,
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

            self.update_user(&msg.from);

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

                    self.update_user(&reply.from);
                }
            }
        }

        Ok(())
    }

    pub fn query(
        self: &mut Self,
        chat_id: i64,
        dates: (i64, i64),
        user_id: i64,
    ) -> QueryResult {
        let mut result = QueryResult {
            start_day: 0,
            daily_users: Vec::new(),
            daily_messages: Vec::new(),
            messages_by_hour: [0; 24],
            messages_by_weekday: [0; 7],
            user_names: Vec::new(),
            messages_by_user: Vec::new(),
        };

        let args: &[(&str, &ToSql)] = &[
            (":chat_id", &chat_id),
            (":day_from", &dates.0),
            (":day_to", &dates.1),
            (":user_id", &user_id),
        ];

        let mut prev_day = 0;
        db_util::query_map_named(
            &mut self.conn,
            "
                SELECT day, COUNT(DISTINCT user_id), SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                   AND day BETWEEN :day_from AND :day_to
                   AND (:user_id = 0 OR :user_id = user_id)
                 GROUP BY day
            ",
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
        );

        db_util::query_map_named(
            &mut self.conn,
            "
                SELECT hour, SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                   AND day BETWEEN :day_from AND :day_to
                   AND (:user_id = 0 OR :user_id = user_id)
                 GROUP BY hour
            ",
            args,
            |row| {
                let hour: i64 = row.get(0);
                result.messages_by_hour[hour as usize] = row.get(1);
            },
        );

        db_util::query_map_named(
            &mut self.conn,
            "
                SELECT (day+4)%7, SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                   AND day BETWEEN :day_from AND :day_to
                   AND (:user_id = 0 OR :user_id = user_id)
                 GROUP BY (day+4)%7
            ",
            args,
            |row| {
                let weekday: i64 = row.get(0);
                result.messages_by_weekday[weekday as usize] = row.get(1);
            },
        );

        db_util::query_map_named(
            &mut self.conn,
            "
                SELECT messages.user_id, users.full_name, SUM(count)
                  FROM messages
                 INNER JOIN users ON users.user_id = messages.user_id
                 WHERE chat_id = :chat_id
                   AND day BETWEEN :day_from AND :day_to
                   AND (:user_id = 0 OR :user_id = messages.user_id)
                 GROUP BY(messages.user_id)
                 ORDER BY SUM(COUNT) DESC
            ",
            args,
            |row| {
                result.user_names.push(row.get(1));
                result.messages_by_user.push(row.get(2));
            },
        );

        result
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
    for _ in 0..32 {
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
