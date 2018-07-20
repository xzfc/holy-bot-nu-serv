use rusqlite;
use std::convert;
use std::error;
use std::fmt;
use std::io;
use reqwest;
use telegram_bot;

#[derive(Debug)]
pub enum MyError {
    SqlError(rusqlite::Error),
    IoError(io::Error),
    ReqwestError(reqwest::Error),
    TelegramError(telegram_bot::Error),
}
impl error::Error for MyError {
    fn description(&self) -> &str {
        "Shit happens"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
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

impl convert::From<reqwest::Error> for MyError {
    fn from(e: reqwest::Error) -> Self {
        MyError::ReqwestError(e)
    }
}

impl convert::From<telegram_bot::Error> for MyError {
    fn from(e: telegram_bot::Error) -> Self {
        MyError::TelegramError(e)
    }
}
