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

pub trait GetMyIpAddr<T> {
    fn get_my_ip_addr(&self) -> Result<T>;
}

impl IpProvider {
    fn build(&self) -> Box<GetMyIpAddr<IpAddr>>  {
        match self {
            &IpProvider::Stdin => Box::new(StdinIpProvider),
            &IpProvider::SfrLaBoxFibre => Box::new(FromRegexIpProvider::new(HttpIpProvider::new(URL_SFR_LABOX_FIBRE))),
            &IpProvider::OpenDNS => Box::new(FromRegexIpProvider::new(HttpIpProvider::new(URL_OPENDNS))),
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

impl GetMyIpAddr<IpAddr> for IpProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        self.build().get_my_ip_addr()
    }
}

//
// stdin
//

struct StdinIpProvider;

impl GetMyIpAddr<IpAddr> for StdinIpProvider {
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

impl<'a> GetMyIpAddr<String> for HttpIpProvider<'a> {
    fn get_my_ip_addr(&self) -> Result<String> {
        let client = Client::new();

        let mut res = try!(client.get(self.url)
            .header(Connection::close())
            .send());

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        trace!("HTTP Response: {}", body);

        Ok(body)
    }
}

pub struct FromRegexIpProvider<P: GetMyIpAddr<String>> {
    provider: P,
}

impl<'a, P: GetMyIpAddr<String>> FromRegexIpProvider<P> {
    fn new(provider: P) -> FromRegexIpProvider<P> {
        FromRegexIpProvider {
            provider: provider,
        }
    }
}

impl<P: GetMyIpAddr<String>> GetMyIpAddr<IpAddr> for FromRegexIpProvider<P> {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        let body = self.provider.get_my_ip_addr().unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;
    use error::Result;
    use std::net::IpAddr;
    use std::str::FromStr;

    static IP_V4: &'static str = "100.3.5.4";

    struct IPv4BodyHP;

    impl GetMyIpAddr<String> for IPv4BodyHP {
        fn get_my_ip_addr(&self) -> Result<String> {
            Ok(IP_V4.to_owned())
        }
    }

    #[test]
    fn ipv4_addr() {
        let mockHP = IPv4BodyHP;
        let provider = FromRegexIpProvider::new(mockHP);
        let maybeResult = provider.get_my_ip_addr();
        assert_eq!(IpAddr::from_str(IP_V4).unwrap(), maybeResult.unwrap());
    }
}
