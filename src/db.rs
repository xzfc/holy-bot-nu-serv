use rusqlite::Connection;
use rusqlite::types::ToSql;
use super::db_util;
use super::error::MyError;
use super::serde_json;

#[derive(Debug, Serialize)]
pub struct QueryResult {
    title: String,
    hours: (i64, i64),

    start_day: i64,
    skip_day: i64,
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
    offset: i64,
    user_id: Option<&str>,
    weekday: Option<u8>,
) -> (u16, String) {
    match query(conn, chat, dates, offset, user_id, weekday) {
        Ok(res) => res,
        Err(e) => (500, format!("Error:\n{:?}", e)),
    }
}

const ERR_INVALID_OFFSET:  &str = r#"{"error":"invalid offset"}"#;
const ERR_INVALID_DATES:   &str = r#"{"error":"invalid dates"}"#;
const ERR_INVALID_WEEKDAY: &str = r#"{"error":"invalid weekday"}"#;
const ERR_CHAT_NOT_FOUND:  &str = r#"{"error":"chat not found"}"#;
const ERR_USER_NOT_FOUND:  &str = r#"{"error":"user not found"}"#;

pub fn query(
    conn: &Connection,
    chat: &str,
    dates: Option<(i64, i64)>,
    offset: i64,
    user_rid: Option<&str>,
    weekday: Option<u8>,
) -> Result<(u16, String), MyError> {

    if offset < -12 || offset > 12 {
        return Ok((400, String::from(ERR_INVALID_OFFSET)));
    }

    if let Some(weekday) = weekday {
        if weekday >= 7 {
            return Ok((400, String::from(ERR_INVALID_WEEKDAY)));
        }
    }

    if let Some((from, to)) = dates {
        if from < 17000 || to < 17000 || to - from > 1000 {
            return Ok((400, String::from(ERR_INVALID_DATES)));
        }
    }

    let (chat_id, chat_title) = match search_chat(conn, chat) {
        Some(x) => x,
        None => return Ok((404, String::from(ERR_CHAT_NOT_FOUND))),
    };

    let mut result = QueryResult {
        title: chat_title,
        hours: (0, 0),

        start_day: 0,
        skip_day: 1,
        daily_users: Vec::new(),
        daily_messages: Vec::new(),

        messages_by_hour: [0; 24],
        messages_by_weekday: [0; 7],

        user_ids: Vec::new(),
        user_names: Vec::new(),
        messages_by_user: Vec::new(),
    };

    let mut _user_id: i64 = 0;
    let mut _weekday: u8 = 0;
    let hours = dates.map(|(a,b)| (a, a*24 - offset, b*24 - offset + 23));
    let mut args: Vec<(&str, &ToSql)> = Vec::new();
    args.push((":chat_id", &chat_id));
    args.push((":offset", &offset));

    let mut filter = String::from("");
    if let Some(user_rid) = user_rid.as_ref() {
        let user_id = match search_user(conn, user_rid) {
            Some(user_id) => user_id,
            None => return Ok((404, String::from(ERR_USER_NOT_FOUND))),
        };

        filter += "AND :user_id = messages.user_id ";
        _user_id = user_id;
        args.push((":user_id",  &_user_id));
    }
    if let Some(hours) = hours.as_ref() {
        filter += "AND hour BETWEEN :hour_from AND :hour_to ";
        result.start_day = hours.0;

        args.push((":hour_from", &hours.1));
        args.push((":hour_to",   &hours.2));
    }
    if let Some(weekday) = weekday {
        filter += "AND (hour + :offset)/24%7 = :weekday";
        _weekday = (weekday + 4)%7;
        result.skip_day = 7;
        if result.start_day != 0 {
            result.start_day += 6 - (result.start_day - weekday as i64 + 2) % 7;
        }
        args.push((":weekday",  &_weekday));
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
                    if let Some(weekday) = weekday {
                        if weekday as i64 != (d+3) % 7 {
                            continue
                        }
                    }
                    result.daily_users.push(0);
                    result.daily_messages.push(0);
                }
            }
            prev_day = day;
            if let Some(weekday) = weekday {
                if weekday as i64 != (day + 3) % 7 {
                    println!("Shit! {} {}", weekday, (day + 3) % 7);
                }
            }
            result.daily_users.push(row.get(1));
            result.daily_messages.push(row.get(2));
        },
    )?;
    if let Some(dates) = dates.as_ref() {
        for d in prev_day..dates.1 {
            if let Some(weekday) = weekday {
                if weekday as i64 != (d+4) % 7 {
                    continue
                }
            }
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

    result.hours = conn.query_row_named(
        "
            SELECT MIN(hour), MAX(hour)
              FROM messages
             WHERE chat_id = :chat_id
        ",
        &[(":chat_id", &chat_id)],
        |row| (row.get::<_, i64>(0), row.get::<_, i64>(1)),
    )?;

    Ok((200, serde_json::to_string(&result).unwrap()))
}

fn search_chat(conn: &Connection, chat: &str) -> Option<(i64, String)> {
    let res = conn.query_row(
        "
            SELECT id, name
              FROM chats
             WHERE alias = ?1
                OR rnd_id = ?1
        ",
        &[&chat],
        |row| (row.get(0), row.get(1)),
    );
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
        |row| row.get::<_, i64>(0),
    );
    match res {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}
