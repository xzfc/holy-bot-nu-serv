use hyper::{Body, Request, Response, Server};
use hyper;
use hyper::rt::Future;
use hyper::service::service_fn_ok;

use db::Db;
use std::sync::Mutex;
use std::sync::Arc;

pub fn run(db: Db) {
    let addr = ([127, 0, 0, 1], 3000).into();
    let db = Arc::new(Mutex::new(db));

    let new_svc = move || {
        let db = db.clone();
        service_fn_ok(move |req| {
            let db = db.lock().unwrap();
            match req.uri().path() {
                "/stats" => {
                    let text = db.query("@caninas", (0, 100000), 0);
                    Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(200)
                        .body(Body::from(text))
                }
                _ => {
                    Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(404)
                        .body(Body::from("404"))
                }
            }.unwrap()
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
