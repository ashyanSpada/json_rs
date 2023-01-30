use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    Message(String),

    InvalidCharInString(usize, char),
    InvalidEscape(usize, char),
    InvalidHexEscape(usize, char),
    InvalidEscapeValue(usize, u32),
    Unexpected(usize, char),
    UnterminatedString(usize),
    EofWhileParsingValue(usize),

    Wanted {
        at: usize,
        expected: char,
        found: char,
    },

    InvalidNumber(String),
    NumberOutOfRange,
    NotSupportedChar(char, usize),

    OpNotExist(String),
    JSONKeyMustBeString(),
    InvalidStructString(),
    InvalidEnumString(),
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Error::Message(msg) => formatter.write_str(&msg),
            _ => formatter.write_str("unexpected end of input"),
        }
    }
}

impl std::error::Error for Error {}
