use rusqlite::{Connection, Row};
use rusqlite::types::ToSql;

pub fn query_map_named<F>(
    conn: &mut Connection,
    sql: &str,
    params: &[(&str, &ToSql)],
    f: F)
where F : FnMut(&Row) -> ()
{
    let mut ff = f; // XXX: WTF?
    let mut stmt = conn.prepare(sql).unwrap();
    let rows = stmt.query_map_named(params, |row| {
        ff(row);
    }).unwrap();
    for _ in rows { }
}
