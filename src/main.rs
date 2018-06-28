// Sqlite
extern crate rusqlite;

// Telega
extern crate telegram_bot_raw;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod db;
mod db_util;

fn main() {
    let mut db = db::Db::new();
    db.init();
    db.update_from_file("/n/Dev2/HolyCrackers/n/identity/data/b2");

    println!("{}", serde_json::to_string(&
            db.query(-1001281121718, (0, 100000), 0)).unwrap()
             );
}
