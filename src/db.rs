use rusqlite::Connection;
use rusqlite::types::ToSql;
use super::db_util;
use super::error::MyError;
use super::serde_json;

#[derive(Debug, Serialize)]
pub struct QueryResult {
    start_day: i64,
    daily_users: Vec<i64>,
    daily_messages: Vec<i64>,

    messages_by_hour: [i64; 24],
    messages_by_weekday: [i64; 7],

    user_ids: Vec<String>,
    user_names: Vec<String>,
    messages_by_user: Vec<i64>,
}

pub fn query_http(
    conn: &Connection,
    chat: &str,
    dates: Option<(i64, i64)>,
    user_id: Option<&str>,
) -> (u16 ,String) {
    match query(conn, chat, dates, 0, user_id) {
        Ok(res) => res,
        Err(e) => (500, format!("Error:\n{:?}", e)),
    }
}

pub fn query(
    conn: &Connection,
    chat: &str,
    dates: Option<(i64, i64)>,
    offset: i64,
    user_rid: Option<&str>,
) -> Result<(u16, String), MyError> {
    let chat_id = match search_chat(conn, chat) {
        Some(chat_id) => chat_id,
        None => {
            return Ok((404, String::from(r#"{"error":"chat not found"}"#)));
        }
    };

    let mut result = QueryResult {
        start_day: 0,
        daily_users: Vec::new(),
        daily_messages: Vec::new(),
        messages_by_hour: [0; 24],
        messages_by_weekday: [0; 7],
        user_ids: Vec::new(),
        user_names: Vec::new(),
        messages_by_user: Vec::new(),
    };

    let mut _user_id: i64 = 0;
    let hours = dates.map(|(a,b)| (a*24 - offset, b*24 - offset));
    let mut args: Vec<(&str, &ToSql)> = Vec::new();
    args.push((":chat_id", &chat_id));
    args.push((":offset", &offset));

    let mut filter = String::from("");
    /*
    if let Some(user_rid) = user_rid.as_ref() {
        let user_id =
            match search_user(conn, user_rid) {
                Some(user_id) => user_id,
                None =>
                    return Ok((
                            404, String::from(
                                r#"{"error":"user not found"}"#))),
            };

        filter += "AND :user_id = messages.user_id ";
        _user_id = user_id;
        args.push((":user_id",  &_user_id));
    }
    */
    if let Some(hours) = hours.as_ref() {
        filter += "AND hour BETWEEN :hour_from AND :hour_to ";
        result.start_day = hours.0 / 24;

        args.push((":hour_from", &hours.0));
        args.push((":hour_to",   &hours.1));
    }
    let args = args.as_slice();

    let mut prev_day = result.start_day - 1;
    db_util::query_map_named(
        &conn,
        format!("
            SELECT (hour + :offset)/24
                 , COUNT(DISTINCT user_id)
                 , SUM(count)
              FROM messages
             WHERE chat_id = :chat_id
                   {}
             GROUP BY (hour + :offset)/24
        ", filter).as_ref(),
        args,
        |row| {
            let day = row.get(0);
            if result.start_day == 0 {
                result.start_day = day;
            } else {
                for d in prev_day + 1..day {
                    result.daily_users.push(0);
                    result.daily_messages.push(0);
                }
            }
            prev_day = day;
            result.daily_users.push(row.get(1));
            result.daily_messages.push(row.get(2));
        },
    )?;
    if let Some(dates) = dates.as_ref() {
        for d in prev_day..dates.1 {
            result.daily_users.push(0);
            result.daily_messages.push(0);
        }
    }

    db_util::query_map_named(
        &conn,
        format!("
            SELECT (hour + :offset) % 24
                 , SUM(count)
              FROM messages
             WHERE chat_id = :chat_id
                   {}
             GROUP BY (hour + :offset) % 24
        ", filter).as_ref(),
        &args,
        |row| {
            let hour: i64 = row.get(0);
            result.messages_by_hour[hour as usize] = row.get(1);
        },
    )?;

    db_util::query_map_named(
        &conn,
        format!("
            SELECT ((hour + :offset)/24 + 3)%7, SUM(count)
              FROM messages
             WHERE chat_id = :chat_id
                   {}
             GROUP BY ((hour + :offset)/24 + 3)%7
        ", filter).as_ref(),
        &args,
        |row| {
            let weekday: i64 = row.get(0);
            result.messages_by_weekday[weekday as usize] = row.get(1);
        },
    )?;

    db_util::query_map_named(
        &conn,
        format!("
            SELECT users.rnd_id
                 , users.name
                 , SUM(messages.count)
                 , :offset -- ignore
              FROM messages
             INNER JOIN users ON users.id = messages.user_id
             WHERE messages.chat_id = :chat_id
                   {}
             GROUP BY(messages.user_id)
             ORDER BY SUM(COUNT) DESC
        ", filter).as_ref(),
        &args,
        |row| {
            result.user_ids.push(row.get(0));
            result.user_names.push(row.get(1));
            result.messages_by_user.push(row.get(2));
        },
    )?;

    return Ok((200, serde_json::to_string(&result).unwrap()))
}

fn search_chat(conn: &Connection, chat: &str) -> Option<i64> {
    let res = conn.query_row(
        "
            SELECT id
              FROM chats
             WHERE alias = ?1
                OR rnd_id = ?1
        ",
        &[&chat],
        |row| row.get::<_,i64>(0));
    match res {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

fn search_user(conn: &Connection, random_id: &str) -> Option<i64> {
    let res = conn.query_row(
        "
            SELECT id
              FROM users
             WHERE rnd_id = ?
        ",
        &[&random_id],
        |row| row.get::<_, i64>(0));
    match res {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}
