use rand::{thread_rng, Rng};
use rusqlite::types::ToSql;
use rusqlite::{Connection, Row, Error};
use super::error::MyError;

pub fn query_map_named<F>(
    conn: &Connection,
    sql: &str,
    params: &[(&str, &ToSql)],
    f: F,
) -> Result<(), MyError>
  where
    F: FnMut(&Row) -> (),
{
    let mut ff = f; // XXX: WTF?
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map_named(params, |row| { ff(row); })?;
    for row in rows { row? }
    Ok(())
}

pub fn query_row<T, F>(
    conn: &Connection,
    sql: &str,
    params: &[&ToSql],
    f: F,
) -> Result<Option<T>, Error>
  where F: FnOnce(&Row) -> T
{
    match conn.query_row(sql, params, f) {
        Ok(x) => Ok(Some(x)),
        Err(Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn random_id() -> String {
    let mut result = String::from("");
    let mut rng = thread_rng();
    for _ in 0..8 {
        let v = (rng.gen::<u32>() % (10+26+26)) as u8;
        let v = v + match v {
            00..=09 => '0' as u8,
            10..=35 => 'a' as u8 - 10,
            36..=61 => 'A' as u8 - 36,
            _ => 0,
        };
        result.push(v as char);
    }
    result
}
