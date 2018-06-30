use rusqlite::{Connection, Row};
use rusqlite::types::ToSql;
use std::process::exit;

pub fn query_map_named<F>(
    conn: &mut Connection,
    sql: &str,
    params: &[(&str, &ToSql)],
    f: F,
) where
    F: FnMut(&Row) -> (),
{
    let mut ff = f; // XXX: WTF?
    let mut stmt = conn.prepare(sql).unwrap();
    let rows = stmt.query_map_named(params, |row| { ff(row); }).unwrap();
    for _ in rows {}
}

pub fn execute_many(
    conn: &mut Connection,
    sqls: &str,
) {
    for sql in sqls.split(";") {
        if sql.trim() == "" {
            continue
        }
        match conn.execute(sql, &[]) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error while executing following statement:");
                eprintln!("\x1b[2m{}\x1b[m", sql.trim());
                eprintln!("\x1b[31m{}\x1b[m", e);
                exit(1);
            }
        }
    }
}
