use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::net::AddrParseError;

use self::Error::{
    Io,
    AddrParse,
};

/// Result type often returned from methods
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    AddrParse(AddrParseError)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Io(ref err) => err.fmt(f),
            AddrParse(ref err) => err.fmt(f),
//            Another => write!(f, "No matching cities with a \
//                                             population were found."),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Io(ref err) => err.description(),
            AddrParse(ref err) => err.description(),
//            Another => "not found",
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Io(err)
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Error {
        AddrParse(err)
    }
}
