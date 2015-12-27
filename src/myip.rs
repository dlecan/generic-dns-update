use ip::IpAddr;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub trait MyIPAddressProvider<'a> {
    fn get_my_ip_addr(&self) -> IpAddr;
}

pub struct StdinIpProvider;

impl<'a> MyIPAddressProvider<'a> for StdinIpProvider {
    fn get_my_ip_addr(&self) -> IpAddr {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                trace!("Input: {}", input);
            }
            Err(error) => error!("error: {}", error),
        }

        IpAddr::from_str(input.trim()).unwrap()
    }
}
