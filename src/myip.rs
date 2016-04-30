use error::Result;
use error::Error;
use std::net::IpAddr;
use hyper::Client;
use hyper::header::Connection;
use std::io;
use std::io::prelude::*;
use std::result::Result as StdResult;
use std::str::FromStr;
use regex::Regex;

// All HTTP IP providers URL
static URL_SFR_LABOX_FIBRE: &'static str = "http://192.168.0.1/";
static URL_OPENDNS: &'static str = "https://diagnostic.opendns.com/myip";

#[derive(Debug)]
pub enum IpProvider {
    Stdin,
    SfrLaBoxFibre,
    OpenDNS,
}

pub trait GetMyIpAddr {
    fn get_my_ip_addr(&self) -> Result<IpAddr>;
}

impl IpProvider {
    fn build(&self) -> Box<GetMyIpAddr>  {
        match self {
            &IpProvider::Stdin => Box::new(StdinIpProvider),
            &IpProvider::SfrLaBoxFibre => Box::new(HttpIpProvider::new(URL_SFR_LABOX_FIBRE)),
            &IpProvider::OpenDNS => Box::new(HttpIpProvider::new(URL_OPENDNS)),
        }
    }
}

impl FromStr for IpProvider {
    type Err = String;

    fn from_str(s: &str) -> StdResult<IpProvider, String> {
        match s {
            "-" => Ok(IpProvider::Stdin),
            "sfrlaboxfibre" => Ok(IpProvider::SfrLaBoxFibre),
            "opendns" => Ok(IpProvider::OpenDNS),
            value => Err(format!("Unknown value for IP provider: {}", value).to_owned()),
        }
    }
}

impl GetMyIpAddr for IpProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        self.build().get_my_ip_addr()
    }
}

//
// stdin
//

struct StdinIpProvider;

impl GetMyIpAddr for StdinIpProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        let mut input = String::new();

        try!(io::stdin().read_line(&mut input));

        trace!("Read stdin: {}", input);

        let ip_addr = try!(IpAddr::from_str(input.trim()));
        Ok(ip_addr)
    }
}

//
// http
//

struct HttpIpProvider<'a> {
    url: &'a str,
}

impl<'a> HttpIpProvider<'a> {
    fn new(url: &'a str) -> HttpIpProvider {
        HttpIpProvider {
            url: url,
        }
    }
}

impl<'a> GetMyIpAddr for HttpIpProvider<'a> {
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
