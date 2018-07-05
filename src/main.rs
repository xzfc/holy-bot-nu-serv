extern crate rusqlite;
extern crate telegram_bot_raw;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate hyper;

use std::env::args;

mod db;
mod db_util;
mod process_log;
mod error;
mod server;

fn main() {
    let args: Vec<String> = args().collect();
    match args.get(1).unwrap_or(&String::new()).as_ref() {
        "sync-tg" => {
            let mut db = db::Db::new(&args[2]);
            db.update_from_file(&args[3]);
        }
        "server" => {
            let mut db = db::Db::new(&args[2]);
            server::run(db);
        }
        "get-chat" => {
            let mut db = db::Db::new(&args[2]);
            println!("{}", db.query(&args[3], (0, 100000), 0));
        }
        _ => {
            eprintln!("Invalid arguments");
        }
    }
}
