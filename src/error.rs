use rusqlite;
use std::convert;
use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MyError {
    SqlError(rusqlite::Error),
    IoError(io::Error),
}
impl error::Error for MyError {
    fn description(&self) -> &str {
        return "Shit happens"
    }
    fn cause(&self) -> Option<&error::Error> {
        return None;
    }
}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl convert::From<io::Error> for MyError {
    fn from(e: io::Error) -> Self {
        MyError::IoError(e)
    }
}

impl convert::From<rusqlite::Error> for MyError {
    fn from(e: rusqlite::Error) -> Self {
        MyError::SqlError(e)
    }
}
