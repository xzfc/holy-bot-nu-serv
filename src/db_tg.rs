use rusqlite::Connection;
use super::db_util;
use super::error::MyError;
use super::process_log;
use super::serde_json;

use telegram_bot_raw::{Update, UpdateKind, User, Integer, MessageOrChannelPost, MessageChat};

pub fn update_from_file(conn: &mut Connection, path: &str) {
    let err = process_log::process_log(path, &mut DbTg{conn:conn});
    eprintln!("err = {:?}", err);
}


fn update(conn: &mut Connection, upd: Update) -> Result<(), MyError> {
    if let UpdateKind::Message(msg) = upd.kind {
        let user_id = update_user(conn, &msg.from)?;
        let chat_id = match update_chat(conn, &msg.chat)? {
            Some(x) => x, None => return Ok(()),
        };

        conn.execute("
            INSERT INTO messages(chat_id, user_id, hour, count)
            VALUES ( ?1, ?2, ?3, 1 )
            ON CONFLICT (chat_id, user_id, hour)
            DO UPDATE SET count = count + 1
        ", &[
            &chat_id,
            &user_id,
            &(msg.date/60/60),
        ])?;

        if let Some(reply) = msg.reply_to_message {
            if let MessageOrChannelPost::Message(reply) = *reply {
                let reply_user_id = update_user(conn, &reply.from)?;
                conn.execute("
                    INSERT INTO replies(chat_id, from_uid, to_uid, count)
                    VALUES ( ?1, ?2, ?3, 1 )
                    ON CONFLICT (chat_id, from_uid, to_uid)
                    DO UPDATE SET count = count + 1
                    ", &[
                        &chat_id,
                        &user_id,
                        &reply_user_id
                    ])?;
            }
        }
    }

    Ok(())
}

fn update_user(conn: &mut Connection, user: &User) -> Result<i64, MyError> {
    let name = match &user.last_name {
        Some(last_name) => format!("{} {}", user.first_name, last_name),
        None => user.first_name.clone(),
    };

    let db_id = db_util::query_row(
        conn, "
            SELECT id
              FROM users
             WHERE kind = 0
               AND ext_id = ?
        ", &[&Integer::from(user.id)],
        |row| row.get::<_, i64>(0))?;

    let db_id = match db_id {
        Some(db_id) => {
            conn.execute("
                UPDATE users
                   SET name = ?
                 WHERE id = ?
                 ", &[&name, &db_id])?;
            db_id
        }
        None => {
            conn.execute("
                INSERT INTO users(kind, ext_id, rnd_id, name)
                VALUES (0, ?, ?, ?)
            ", &[&Integer::from(user.id), &db_util::random_id(), &name])?;
            conn.last_insert_rowid()
        }
    };

    Ok(db_id)
}

fn update_chat(conn: &mut Connection, chat: &MessageChat) -> Result<Option<i64>, MyError> {
    let (tg_id, title, username) = match &chat {
        MessageChat::Private(_) => return Ok(None),
        MessageChat::Unknown(_) => return Ok(None),
        MessageChat::Group(c)   =>
            (Integer::from(c.id), &c.title, &None),
        MessageChat::Supergroup(c) =>
            (Integer::from(c.id), &c.title, &c.username),
    };
    let username = username.clone().map(|x| format!("@{}", x));

    let db_id = db_util::query_row(
        conn, "
            SELECT id
              FROM chats
             WHERE kind = 0
               AND ext_id = ?
        ", &[&tg_id],
        |row| row.get::<_, i64>(0))?;

    let db_id = match db_id {
        Some(db_id) => {
            conn.execute("
                UPDATE chats
                   SET name = ?
                     , alias = ?
                 WHERE id = ?
                 ", &[title, &username, &db_id])?;
            db_id
        }
        None => {
            conn.execute("
                INSERT INTO chats(kind, ext_id, rnd_id, name, alias)
                VALUES (0, ?, ?, ?, ?)
            ", &[&tg_id, &db_util::random_id(), title, &username])?;
            conn.last_insert_rowid()
        }
    };

    Ok(Some(db_id))
}


struct DbTg<'a> {
    conn: &'a mut Connection,
}

impl<'a> process_log::LogProcessor for DbTg<'a> {
    type Error = MyError;
    fn begin(&mut self) -> Result<Option<u64>, Self::Error> {
        self.conn.execute("BEGIN", &[])?;
        let seek = db_util::query_row(
            self.conn,
            "SELECT value FROM kv WHERE name = 'telegram_seek'",
            &[],
            |row| row.get::<_,i64>(0))?;
        match seek {
            Some(value) => Ok(Some(value as u64)),
            None => Ok(None),
        }
    }
    fn commit(&mut self, end_pos: u64) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO kv VALUES ('telegram_seek', ?)",
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
            Ok(upd) => { update(self.conn, upd) },
            Err(err) => {
                eprintln!("Line: {}\nParse error: {}\n", line, err);
                Ok(())
            }
        }
    }
}
