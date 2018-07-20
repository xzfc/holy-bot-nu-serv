use rusqlite::{Connection};
use super::db_util;
use super::error::MyError;
use super::serde_json;
use telegram_bot::Api;
use telegram_bot::Error;
use telegram_bot::types::requests::{GetUserProfilePhotos, GetFile};
use telegram_bot::types::{UserId, FileRef, PhotoSize};
use telegram_bot_raw::UserProfilePhotos;
use tokio_core::reactor::Core;
use hyper::rt::Future;
use futures::done;
use reqwest;
use std::io::copy;
use std::fs::File;
use std::fs;

#[derive(Debug)]
struct Row {
    id:      i64,
    rnd_id: String,
    tg_id:   i64,
    old:     Option<String>,
}

fn db_get_rows(conn: &mut Connection) -> Result<Vec::<Row>, MyError> {
    let mut rows = Vec::new();
    db_util::query_map_named(
        conn,
        "
            SELECT users.id, users.rnd_id, users.ext_id, users_tg.doc
              FROM users
             LEFT JOIN users_tg
                    ON users.id = users_tg.id
             WHERE users.kind = 0
               AND users_tg.last_upd IS NULL
                OR users_tg.last_upd + 60*60*24 > +strftime('%s', 'now');
        ",
        &[],
        |row| {
            rows.push(Row {
                id:     row.get(0),
                rnd_id: row.get(1),
                tg_id:  row.get(2),
                old:    row.get(3),
            })
        },
    )?;
    println!("Done");
    Ok(rows)
}

fn db_set_have(
    conn: &mut Connection,
    id: i64,
    doc: Option<String>
) -> Result<(), MyError>
{
    conn.execute(
        "
            INSERT OR REPLACE INTO users_tg(id, last_upd, doc)
            VALUES (?, +strftime('%s', 'now'), ?)
        ",
        &[&id, &doc]
    )?;
    Ok(())
}

fn getFile(file_id: String) -> GetFile {
    GetFile::new(
        PhotoSize {
            file_id: file_id,
            width: 0,
            height: 0,
            file_size: None
        }
    )
}

fn saveToFile(
    token: &str,
    file_path: &str,
    save_path: &str,
) -> Result<(), MyError> {
    let url = format!("https://api.telegram.org/file/bot{}/{}", token, file_path);
    let mut resp = reqwest::get(url.as_str())?;
    assert!(resp.status().is_success());

    let out = format!("./ava/{}.jpg", save_path);
    let mut file = File::create(out)?;
    copy(&mut resp, &mut file)?;
    Ok(())
}

fn yoba3 (
    conn: &mut Connection,
    core: &mut Core,
    api: &Api,
    token: &str,
    row: &Row,
) -> Result<(), MyError> {
    let new =
        core.run(
            api.send(GetUserProfilePhotos::new(UserId::from(row.tg_id)).limit(1))
        );
    let new = match new {
        Ok(x) => x,
        Err(x) => {
            db_set_have(conn, row.id, None)?;
            return Ok(())
        },
    };
    let new = new
        .photos.get(0)
        .and_then(|x|x.last())
        .map(|x|x.file_id.clone());

    if new != row.old {
        if let Some(ref new) = new {
            let file_path =
                core.run(
                    api.send(getFile(new.to_string()))
                    .map(|file| { file.file_path.unwrap() })
                ).unwrap();
            saveToFile(token, &file_path, &row.rnd_id);
        } else {
            fs::remove_file(format!("./ava/{}.jpg", row.rnd_id));
        }
        db_set_have(conn, row.id, new)?;
    } else {
    }
    Ok(())
}

pub fn update(
    conn: &mut Connection,
    token: &str,
) -> Result<(), MyError> {
    let mut core = Core::new().unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    for row in db_get_rows(conn)?.iter() {
        println!("{:?}", row);
        yoba3(conn, &mut core, &api, token, row)?;
    }

    Ok(())
}
