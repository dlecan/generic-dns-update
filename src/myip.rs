use config::Config;
use error::Result;
use error::Error;
use ip::IpAddr;
use hyper::Client;
use hyper::header::Connection;
use std::io;
use std::io::prelude::*;
use std::result::Result as StdResult;
use std::str::FromStr;
use regex::Regex;

// All HTTP IP providers URL
static URL_SFR_LABOX_FIBRE: &'static str = "http://192.168.0.1/";

#[derive(Debug)]
pub enum IpProvider {
    Stdin,
    SfrLaBoxFibre,
}

impl FromStr for IpProvider {
    type Err = String;

    fn from_str(s: &str) -> StdResult<IpProvider, String> {
        match s {
            "-" => Ok(IpProvider::Stdin),
            "sfrlaboxfibre" => Ok(IpProvider::SfrLaBoxFibre),
            value => Err(format!("Unknown value for IP provider: {}", value).to_owned()),
        }
    }
}

pub trait IpAddressProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr>;
}

impl IpAddressProvider {
    pub fn from_config(config: &Config) -> Box<IpAddressProvider + 'static>  {
        match config.ip_provider {
            IpProvider::Stdin => Box::new(StdinIpProvider),
            IpProvider::SfrLaBoxFibre => Box::new(HttpIpProvider::new(URL_SFR_LABOX_FIBRE)),
        }
    }
}

pub struct StdinIpProvider;

impl IpAddressProvider for StdinIpProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        let mut input = String::new();

        try!(io::stdin().read_line(&mut input));

        trace!("Read stdin: {}", input);

        let ip_addr = try!(IpAddr::from_str(input.trim()));
        Ok(ip_addr)
    }
}

pub struct HttpIpProvider<'a> {
    url: &'a str,
}

impl<'a> HttpIpProvider<'a> {
    fn new(url: &'a str) -> HttpIpProvider {
        HttpIpProvider {
            url: url,
        }
    }
}

impl<'a> IpAddressProvider for HttpIpProvider<'a> {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        let client = Client::new();

        let mut res = try!(client.get(self.url)
            .header(Connection::close())
            .send());

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        trace!("HTTP Response: {}", body);

        let regex = try!(Regex::new(r"((?:(?:0|1[\d]{0,2}|2(?:[0-4]\d?|5[0-5]?|[6-9])?|[3-9]\d?)\.){3}(?:0|1[\d]{0,2}|2(?:[0-4]\d?|5[0-5]?|[6-9])?|[3-9]\d?))"));

        regex.captures(&*body)
            .and_then(|caps| caps.at(1))
            // Convert Option to Result
            .ok_or(Error::IpNotFound)
            .and_then(|val| IpAddr::from_str(val).map_err(|e| {
                error!("IP parse error: {}", e);
                Error::IpNotFound
            }))
    }
}
