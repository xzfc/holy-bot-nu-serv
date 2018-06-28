// Sqlite
extern crate rusqlite;
mod db_util;
use rusqlite::Connection;

// Telega
extern crate telegram_bot_raw;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use telegram_bot_raw::{Update,UpdateKind, User, Integer, MessageOrChannelPost};

struct Db {
    conn: Connection,
}

#[derive(Debug, Serialize)]
struct QueryResult {
    start_day: i64,
    daily_users: Vec<i64>,
    daily_messages: Vec<i64>,

    messages_by_hour: [i64; 24],
    messages_by_weekday: [i64; 7],

    user_names: Vec<String>,
    messages_by_user: Vec<i64>,
}

impl Db {
    fn new() -> Self {
        Db {
            conn: Connection::open("1.db").unwrap(),
        }
    }

    fn init(self: &mut Self) {
        self.conn.execute("
            CREATE TABLE IF NOT EXISTS messages (
                chat_id INTEGER,
                user_id INTEGER,
                day     INTEGER,
                hour    INTEGER,
                count   INTEGER,
                PRIMARY KEY (chat_id, user_id, day, hour)
            )", &[]).unwrap();

        self.conn.execute("
            CREATE INDEX IF NOT EXISTS messages_i0
            ON messages ( chat_id, day )
            ", &[]).unwrap();

        self.conn.execute("
            CREATE INDEX IF NOT EXISTS messages_i1
            ON messages ( chat_id, (day+4) % 7 )
            ", &[]).unwrap();

        self.conn.execute("
            CREATE TABLE IF NOT EXISTS users (
                user_id   INTEGER,
                full_name TEXT,
                PRIMARY KEY (user_id)
            )", &[]).unwrap();

        self.conn.execute("
            CREATE TABLE IF NOT EXISTS replies (
                chat_id  INTEGER,
                uid_from INTEGER,
                uid_to   INTEGER,
                count    INTEGER,
                PRIMARY KEY (chat_id, uid_from, uid_to)
            )", &[]).unwrap();
    }

    fn begin(self: &mut Self) {
        self.conn.execute("BEGIN", &[]).unwrap();
    }

    fn commit(self: &mut Self) {
        self.conn.execute("COMMIT", &[]).unwrap();
    }

    fn update_user(self: &mut Self, user: &User) {
        let full_name = match &user.last_name {
            Some(last_name) => format!("{} {}", user.first_name, last_name),
            // XXX: actually, cloning here is redundant
            None => user.first_name.clone(),
        };
        self.conn.execute("
            INSERT OR REPLACE INTO users VALUES
            ( ?1, ?2 )
        ", &[&Integer::from(user.id), &full_name]).unwrap();
    }

    fn update(self: &mut Self, upd: Update) {
        if let UpdateKind::Message(msg) = upd.kind {
            self.conn.execute("
                INSERT INTO messages VALUES
                ( ?1, ?2, ?3, ?4, 1 )
                ON CONFLICT (chat_id, user_id, day, hour)
                DO UPDATE SET count = count + 1
            ", &[&Integer::from(msg.chat.id()),
                 &Integer::from(msg.from.id),
                 &(msg.date/60/60/24),
                 &(msg.date/60/60%24)]).unwrap();

            self.update_user(&msg.from);

            if let Some(reply) = msg.reply_to_message {
                if let MessageOrChannelPost::Message(reply) = *reply {
                    self.conn.execute("
                        INSERT INTO replies VALUES
                        ( ?1, ?2, ?3, 1 )
                        ON CONFLICT (chat_id, uid_from, uid_to)
                        DO UPDATE SET count = count + 1
                    ", &[&Integer::from(msg.chat.id()),
                         &Integer::from(msg.from.id),
                         &Integer::from(reply.from.id)]).unwrap();

                    self.update_user(&reply.from);
                }
            }
        }
    }

    fn query(self: &mut Self,
             chat_id: i64,
             dates: (i64, i64),
             user_id: i64) -> QueryResult {
        let mut result = QueryResult {
            start_day: 0,
            daily_users: Vec::new(),
            daily_messages: Vec::new(),
            messages_by_hour: [0; 24],
            messages_by_weekday: [0; 7],
            user_names: Vec::new(),
            messages_by_user: Vec::new(),
        };

        let args: &[(&str, &rusqlite::types::ToSql)] =
            &[(":chat_id", &chat_id),
              (":day_from", &dates.0),
              (":day_to", &dates.1),
              (":user_id", &user_id)];

        let mut prev_day = 0;
        db_util::query_map_named(
            &mut self.conn, "
            SELECT day, COUNT(DISTINCT user_id), SUM(count)
              FROM messages
             WHERE chat_id = :chat_id
               AND day BETWEEN :day_from AND :day_to
               AND (:user_id = 0 OR :user_id = user_id)
             GROUP BY day
            ", args, |row| {
                let day = row.get(0);
                if result.start_day == 0 {
                    result.start_day = day;
                } else {
                    for _ in prev_day+1..day {
                        result.daily_users.push(0);
                        result.daily_messages.push(0);
                    }
                }
                prev_day = day;
                result.daily_users.push(row.get(1));
                result.daily_messages.push(row.get(2));
            });

        db_util::query_map_named(
            &mut self.conn, "
            SELECT hour, SUM(count)
              FROM messages
             WHERE chat_id = :chat_id
               AND day BETWEEN :day_from AND :day_to
               AND (:user_id = 0 OR :user_id = user_id)
             GROUP BY hour
            ", args, |row| {
                let hour: i64 = row.get(0);
                result.messages_by_hour[hour as usize] = row.get(1);
            });

        db_util::query_map_named(
            &mut self.conn, "
                SELECT (day+4)%7, SUM(count)
                  FROM messages
                 WHERE chat_id = :chat_id
                   AND day BETWEEN :day_from AND :day_to
                   AND (:user_id = 0 OR :user_id = user_id)
                 GROUP BY (day+4)%7
            ", args, |row| {
                let weekday: i64 = row.get(0);
                result.messages_by_weekday[weekday as usize] = row.get(1);
            });

        db_util::query_map_named(
            &mut self.conn, "
                SELECT messages.user_id, users.full_name, SUM(count)
                  FROM messages
                 INNER JOIN users ON users.user_id = messages.user_id
                 WHERE chat_id = :chat_id
                   AND day BETWEEN :day_from AND :day_to
                   AND (:user_id = 0 OR :user_id = messages.user_id)
                 GROUP BY(messages.user_id)
                 ORDER BY SUM(COUNT) DESC
            ", &[(":chat_id", &chat_id),
                 (":day_from", &dates.0),
                 (":day_to", &dates.1),
                 (":user_id", &user_id)],
            |row| {
                result.user_names.push(row.get(1));
                result.messages_by_user.push(row.get(2));
            });

        result
    }
}

fn update_from_file(db: &mut Db) {
    db.begin();

    let f = File::open("/n/Dev2/HolyCrackers/n/identity/data/b2").unwrap();
    let file = BufReader::new(&f);
    let mut n = 0;
    for line in file.lines() {
        let line = line.unwrap();
        let a  = serde_json::from_str::<Update>(&line);
        n += 1;
        if n % 1000 == 0 { println!("{}", n); }
        match a {
            Ok(upd) => db.update(upd),
            Err(err) => println!("Line: {}\nError: {}\n", line, err)
        }
    }

    db.commit();
}

fn main() {
    let mut db = Db::new();
    db.init();
    update_from_file(&mut db);

    println!("{}", serde_json::to_string(&
            db.query(-1001281121718, (0, 100000), 0)).unwrap()
             );
}
