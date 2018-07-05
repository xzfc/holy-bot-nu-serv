use rusqlite::{Connection, Row};
use rusqlite::types::ToSql;
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
