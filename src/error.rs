use std::error::Error;
use std::fmt;
use std::io;
use std::net::AddrParseError;

#[derive(Debug)]
pub enum IpError {
    Io(io::Error),
    AddrParse(AddrParseError)
}

impl fmt::Display for IpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpError::Io(ref err) => err.fmt(f),
            IpError::AddrParse(ref err) => err.fmt(f),
//            IpError::Another => write!(f, "No matching cities with a \
//                                             population were found."),
        }
    }
}

impl Error for IpError {
    fn description(&self) -> &str {
        match *self {
            IpError::Io(ref err) => err.description(),
            IpError::AddrParse(ref err) => err.description(),
//            IpError::Another => "not found",
        }
    }
}

impl From<io::Error> for IpError {
    fn from(err: io::Error) -> IpError {
        IpError::Io(err)
    }
}

impl From<AddrParseError> for IpError {
    fn from(err: AddrParseError) -> IpError {
        IpError::AddrParse(err)
    }
}
