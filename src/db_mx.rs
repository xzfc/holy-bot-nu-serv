use rusqlite::{Connection, Error};
use std::fs::File;
use std::io::Read;
use super::error::MyError;
use super::serde_json;

pub fn update_from_file(conn: &mut Connection, path: &str) {
    conn.execute("BEGIN", &[]).unwrap();
    match update_from_file_inner(conn, path) {
        Ok(_) => {
            conn.execute("COMMIT", &[]).unwrap();
        }
        Err(_) => {
            conn.execute("ABORT", &[]).unwrap();
        }
    }
}

fn update_from_file_inner(
    conn: &mut Connection,
    path: &str,
) -> Result<(), MyError> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let a = serde_json::from_str::<serde_json::Value>(&contents).unwrap();
    update(conn, &a)?;
    Ok(())
}

fn update(conn: &mut Connection, val: &serde_json::Value) -> Result<(), Error> {
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
            .and_then(|x| x.as_array())
    );
    for it in chunk.iter() {
        let it = try_or!(continue, it.as_object());
        let time = try_or!(
            continue,
            it.get("origin_server_ts").and_then(|x| x.as_i64())
        );
        let mxid = try_or!(
            continue,
            it.get("sender").and_then(|x| x.as_str())
        );
        update_item(conn, time, mxid)?;
    }
    Ok(())
}

fn update_item(
    conn: &mut Connection,
    time: i64,
    mxid: &str,
) -> Result<(), Error> {
    let user = conn.query_row(
        "
            SELECT user_id
              FROM users
             WHERE random_id = ?
        ",
        &[&mxid],
        |row| row.get::<_, i64>(0),
    );
    let user_id = match user {
        Ok(x) => x,
        Err(_) => {
            conn.execute(
                "
                    INSERT INTO users
                    VALUES ((SELECT COALESCE(min(user_id)-1, -1)
                               FROM users
                              WHERE user_id < 0), ?1, ?1);
                ",
                &[&mxid],
            )?;
            conn.query_row(
                "
                    SELECT user_id
                      FROM users
                     WHERE random_id = ?
                ",
                &[&mxid],
                |row| row.get::<_, i64>(0),
            )?
        }
    };

    conn.execute(
        "
            INSERT INTO
            messages (chat_id, user_id, day, hour, count)
            VALUES ( 1, ?1, ?2, ?3, 1 )
            ON CONFLICT (chat_id, user_id, day, hour)
            DO UPDATE SET count = count + 1
        ",
        &[
            &user_id,
            &(time/1000/60/60/24),
            &(time/1000/60/60%24),
        ])?;

    Ok(())
}
