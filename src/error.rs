use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::net::AddrParseError;
use std::num::ParseIntError;
use hyper::error::Error as HyperError;
use regex::Error as RegexError;

use self::Error::{
    Io,
    AddrParse,
    XmlRpc,
    Http,
    Regex,
    IpNotFound,
};

/// Result type often returned from methods
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    AddrParse(AddrParseError),
    XmlRpc(String),
    Http(HyperError),
    Regex(RegexError),
    IpNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Io(ref err) => err.fmt(f),
            AddrParse(ref err) => err.fmt(f),
            XmlRpc(ref label) => f.write_str(label),
            Http(ref err) => err.fmt(f),
            Regex(ref err) => err.fmt(f),
            IpNotFound => write!(f, "IP address not found."),
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
            XmlRpc(ref err) => err,
            Http(ref err) => err.description(),
            Regex(ref err) => err.description(),
            IpNotFound => "Ip not found",
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

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        XmlRpc(err.description().to_string())
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Http(err)
    }
}

impl From<RegexError> for Error {
    fn from(err: RegexError) -> Error {
        Regex(err)
    }
}
