use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::net::AddrParseError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    AddrParse(AddrParseError)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::AddrParse(ref err) => err.fmt(f),
//            Error::Another => write!(f, "No matching cities with a \
//                                             population were found."),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::AddrParse(ref err) => err.description(),
//            Error::Another => "not found",
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Error {
        Error::AddrParse(err)
    }
}
