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
mod db_tg;
use rusqlite::Connection;

fn main() {
    let args: Vec<String> = args().collect();
    match args.get(1).unwrap_or(&String::new()).as_ref() {
        "sync-tg" => {
            let mut conn = Connection::open(&args[2]).unwrap();
            db_tg::update_from_file(&mut conn, &args[3]);
        }
        "sync-mx" => {
            let mut conn = Connection::open(&args[2]).unwrap();
            db_mx::update_from_file(&mut conn, &args[3]);
        }
        "server" => {
            let mut conn = Connection::open(&args[2]).unwrap();
            server::run(conn);
        }
        "get-chat" => {
            let mut conn = Connection::open(&args[2]).unwrap();
            match db::query(&conn, &args[3], None, 0, None) {
                Ok((status, res)) => println!("Status: {}\n{}", status, res),
                Err(err) => println!("Error:\n{:?}", err),
            }
        }
        _ => {
            eprintln!("Invalid arguments");
        }
    }
}
