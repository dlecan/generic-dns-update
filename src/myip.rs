use ip::IpAddr;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(Debug)]
enum IpError {
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

pub trait MyIPAddressProvider<'a> {
    fn get_my_ip_addr(&self) -> Result<IpAddr, IpError>;
}

pub struct StdinIpProvider;

impl<'a> MyIPAddressProvider<'a> for StdinIpProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr, IpError> {
        let mut input = String::new();

        try!(io::stdin().read_line(&mut input));

        trace!("Read stdin: {}", input);

        let ip_addr = try!(IpAddr::from_str(input.trim()));
        Ok(ip_addr)
    }
}
