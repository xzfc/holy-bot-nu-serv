extern crate hyper;
extern crate rand;
extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate telegram_bot_raw;
extern crate url;

use std::env::args;

mod db;
mod db_mx;
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
        "sync-mx" => {
            let mut db = db_mx::Db::new(&args[2]);
            db.update_from_file(&args[3]);
        }
        "server" => {
            let mut db = db::Db::new(&args[2]);
            server::run(db);
        }
        "get-chat" => {
            let mut db = db::Db::new(&args[2]);
            match db.query_inner(&args[3], None, None) {
                Ok((status, res)) => println!("Status: {}\n{}", status, res),
                Err(err) => println!("Error:\n{:?}", err),
            }
        }
        _ => {
            eprintln!("Invalid arguments");
        }
    }
}
