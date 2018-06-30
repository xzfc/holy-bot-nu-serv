extern crate rusqlite;
extern crate telegram_bot_raw;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;

mod db;
mod db_util;
mod process_log;
mod error;

fn main() {
    let mut db = db::Db::new("1.db");
    db.init();
    db.update_from_file("/n/Dev2/HolyCrackers/n/identity/data/b2");

    println!(
        "{}",
        serde_json::to_string(&db.query(-1001103425247, (0, 100000), 0))
            .unwrap()
    );
}
