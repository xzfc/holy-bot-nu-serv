use rusqlite::Connection;
//use rusqlite::types::ToSql;
use super::error::MyError;
use super::serde_json;

use std::io::Read;
use std::fs::File;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(path: &str) -> Self {
        Db { conn: Connection::open(path).unwrap() }
    }

    pub fn update_from_file(self: &mut Self, path: &str) {
        self.conn.execute("BEGIN", &[]).unwrap();
        match self.update_from_file_inner(path) {
            Ok(_) => {
                self.conn.execute("COMMIT", &[]).unwrap();
            }
            Err(_) => {
                self.conn.execute("ABORT", &[]).unwrap();
            }
        }
    }

    fn update_from_file_inner(self: &mut Self, path: &str) -> Result<(), MyError> {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let a = serde_json::from_str::<serde_json::Value>(&contents).unwrap();
        self.update(&a)?;
        Ok(())
    }

    fn update(self: &mut Self, val: &serde_json::Value) -> Result<(), MyError> {
        macro_rules! try_or {
            ($fail:expr, $e:expr) => {
                match $e {
                    Some(x) => x,
                    None => $fail,
                }
            };
        }

        let chunk = try_or!(
            return Ok(()),
            val
                .as_object()
                .and_then(|x| x.get("chunk"))
                .and_then(|x| x.as_array()));
        for it in chunk.iter() {
            let it = try_or!(continue, it.as_object());
            let time = try_or!(
                continue,
                it
                    .get("origin_server_ts")
                    .and_then(|x| x.as_i64()));
            let mxid = try_or!(
                continue,
                it
                    .get("sender")
                    .and_then(|x| x.as_str()));
            self.update_item(time, mxid)?;
        }
        Ok(())
    }

    fn update_item(&mut self, time: i64, mxid: &str) -> Result<(), MyError> {
        let user = self.conn.query_row("
                SELECT user_id
                  FROM users
                 WHERE random_id = ?
            ", &[&mxid],
            |row| row.get::<_, i64>(0));
        let user_id = match user {
            Ok(x) => x,
            Err(_) => {
                self.conn.execute("
                        INSERT INTO users
                        VALUES ((SELECT COALESCE(min(user_id)-1, -1)
                                   FROM users
                                  WHERE user_id < 0), ?1, ?1);
                    ", &[&mxid])?;
                self.conn.query_row("
                        SELECT user_id
                          FROM users
                         WHERE random_id = ?
                    ", &[&mxid],
                    |row| row.get::<_, i64>(0))?
            }
        };

        self.conn.execute("
            INSERT INTO
            messages (chat_id, user_id, day, hour, count)
            VALUES ( 1, ?1, ?2, ?3, 1 )
            ON CONFLICT (chat_id, user_id, day, hour)
            DO UPDATE SET count = count + 1
            ", &[
                &user_id,
                &(time/1000/60/60/24),
                &(time/1000/60/60%24),
            ])?;

        Ok(())
    }
}
