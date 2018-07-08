use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use hyper;
use rusqlite::Connection;
use std::sync::Arc;
use std::sync::Mutex;
use super::db;

use url::form_urlencoded;

struct StatsArgs<'a> {
    chat: &'a str,
    dates: Option<(i64, i64)>,
    offset: i64,
    user: Option<String>,
}

enum Args<'a> {
    Stats(StatsArgs<'a>),
    Unknown,
    Invalid,
}

fn parse_stats<'a>(uri: &'a hyper::Uri) -> Args<'a> { 
    macro_rules! try2 {
        ($e:expr) => {
            match $e {
                Ok(x) => x,
                Err(_) => return Args::Invalid,
            }
        };
    }

    let query: form_urlencoded::Parse = form_urlencoded::parse(
        uri.query().unwrap_or("").as_bytes()
    );

    let segments : Vec<&'a str> = uri.path()[1..].split('/').collect();

    if segments.len() == 2 && segments[0] == "stats" {
        let mut from = None;
        let mut to = None;
        let mut offset = None;
        let mut user: Option<String> = None;
        for (key, val) in query {
            match &*key {
                "from"   => from   = Some(try2!(val.parse())),
                "to"     => to     = Some(try2!(val.parse())),
                "offset" => offset = Some(try2!(val.parse())),
                "user"   => user   = Some(val.to_owned().to_string()),
                _ => return Args::Invalid,
            }
        }

        let dates = match (from, to) {
            (Some(from), Some(to)) => Some((from, to)),
            (None, None) => None,
            _ => return Args::Invalid,
        };

        return Args::Stats(StatsArgs {
            chat: segments[1],
            dates: dates,
            offset: offset.unwrap_or(0),
            user: user,
        })
    }

    return Args::Unknown
}

pub fn run(conn: Connection) {
    let addr = ([127, 0, 0, 1], 3000).into();
    let conn = Arc::new(Mutex::new(conn));

    let new_svc = move || {
        let conn = conn.clone();
        service_fn_ok(move |req| {
            let conn = conn.lock().unwrap();
            match parse_stats(req.uri()) {
                Args::Stats(x) => {
                    let (status, text) = db::query_http(
                        &conn,
                        x.chat, x.dates, x.offset,
                        x.user.as_ref().map(|x| &**x),
                        );
                    Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(status)
                        .body(Body::from(text))
                }
                Args::Unknown => 
                    Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(404)
                        .body(Body::from("404")),
                Args::Invalid => 
                    Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(400)
                        .body(Body::from("400")),
            }.unwrap()
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
