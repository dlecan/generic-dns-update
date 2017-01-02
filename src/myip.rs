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
static URL_IPIFY: &'static str = "https://api.ipify.org/";

#[derive(Debug)]
pub enum IpProvider {
    Stdin,
    SfrLaBoxFibre,
    OpenDNS,
    Ipify,
}

pub trait GetMyIpAddr<T> {
    fn get_my_ip_addr(&self) -> Result<T>;
}

impl IpProvider {
    fn build(&self) -> Box<GetMyIpAddr<IpAddr>> {
        match self {
            &IpProvider::Stdin => Box::new(StdinIpProvider),
            &IpProvider::SfrLaBoxFibre => {
                Box::new(FromRegexIpProvider::new(HttpIpProvider::new(URL_SFR_LABOX_FIBRE)))
            }
            &IpProvider::OpenDNS => {
                Box::new(FromRegexIpProvider::new(HttpIpProvider::new(URL_OPENDNS)))
            }
            &IpProvider::Ipify => {
                Box::new(FromRegexIpProvider::new(HttpIpProvider::new(URL_IPIFY)))
            }
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
            "ipify" => Ok(IpProvider::Ipify),
            value => Err(format!("Unknown value for IP provider: {}", value).to_owned()),
        }
    }
}

impl GetMyIpAddr<IpAddr> for IpProvider {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        self.build().get_my_ip_addr()
    }
}

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

// http
//

struct HttpIpProvider<'a> {
    url: &'a str,
}

impl<'a> HttpIpProvider<'a> {
    fn new(url: &'a str) -> HttpIpProvider {
        HttpIpProvider { url: url }
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
        FromRegexIpProvider { provider: provider }
    }
}

impl<P: GetMyIpAddr<String>> GetMyIpAddr<IpAddr> for FromRegexIpProvider<P> {
    fn get_my_ip_addr(&self) -> Result<IpAddr> {
        let body = self.provider.get_my_ip_addr().unwrap();

        let ipv4_regex = try!(Regex::new(r"((?:(?:0|1[\d]{0,2}|2(?:[0-4]\d?|5[0-5]?|[6-9])?|[3-9]\d?)\.){3}(?:0|1[\d]{0,2}|2(?:[0-4]\d?|5[0-5]?|[6-9])?|[3-9]\d?))"));
        let ipv6_regex = try!(Regex::new(r"((([0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){6}:[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){5}:([0-9A-Fa-f]{1,4}:)?[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){4}:([0-9A-Fa-f]{1,4}:){0,2}[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){3}:([0-9A-Fa-f]{1,4}:){0,3}[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){2}:([0-9A-Fa-f]{1,4}:){0,4}[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){6}((\d((25[0-5])|(1\d{2})|(2[0-4]\d)|(\d{1,2}))\d)\.){3}(\d((25[0-5])|(1\d{2})|(2[0-4]\d)|(\d{1,2}))\d))|(([0-9A-Fa-f]{1,4}:){0,5}:((\d((25[0-5])|(1\d{2})|(2[0-4]\d)|(\d{1,2}))\d)\.){3}(\d((25[0-5])|(1\d{2})|(2[0-4]\d)|(\d{1,2}))\d))|(::([0-9A-Fa-f]{1,4}:){0,5}((\d((25[0-5])|(1\d{2})|(2[0-4]\d)|(\d{1,2}))\d)\.){3}(\d((25[0-5])|(1\d{2})|(2[0-4]\d)|(\d{1,2}))\d))|([0-9A-Fa-f]{1,4}::([0-9A-Fa-f]{1,4}:){0,5}[0-9A-Fa-f]{1,4})|(::([0-9A-Fa-f]{1,4}:){0,6}[0-9A-Fa-f]{1,4})|(([0-9A-Fa-f]{1,4}:){1,7}:))"));

        let maybe_ipv4 = ipv4_regex.captures(&*body)
            .and_then(|caps| caps.at(1));


        let maybe_ipv6 = ipv6_regex.captures(&*body)
            .and_then(|caps| caps.at(1));

        maybe_ipv4.or(maybe_ipv6)
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
    use error::Error;
    use error::Result;
    use std::net::IpAddr;
    use std::str::FromStr;

    static IP_V4: &'static str = "100.3.5.4";
    static IP_V6: &'static str = "2a01:ca07:835a:3210:2cdb:dd10:101d:3117";

    struct IPv4BodyHP;

    impl GetMyIpAddr<String> for IPv4BodyHP {
        fn get_my_ip_addr(&self) -> Result<String> {
            Ok(IP_V4.to_owned())
        }
    }

    #[test]
    fn ipv4_addr() {
        let provider = FromRegexIpProvider::new(IPv4BodyHP);
        let maybeResult = provider.get_my_ip_addr();
        assert_eq!(IpAddr::from_str(IP_V4).unwrap(), maybeResult.unwrap());
    }

    struct IPv6BodyHP;

    impl GetMyIpAddr<String> for IPv6BodyHP {
        fn get_my_ip_addr(&self) -> Result<String> {
            Ok(IP_V6.to_owned())
        }
    }

    #[test]
    fn ipv6_addr() {
        let provider = FromRegexIpProvider::new(IPv6BodyHP);
        let maybeResult = provider.get_my_ip_addr();
        assert_eq!(IpAddr::from_str(IP_V6).unwrap(), maybeResult.unwrap());
    }

    struct NotIPAddrBodyHP;

    impl GetMyIpAddr<String> for NotIPAddrBodyHP {
        fn get_my_ip_addr(&self) -> Result<String> {
            Ok("not an ip addr".to_owned())
        }
    }

    #[test]
    fn not_an_ip_addr() {
        let provider = FromRegexIpProvider::new(NotIPAddrBodyHP);
        let maybeResult = provider.get_my_ip_addr();
        assert!(maybeResult.is_err());
    }

}
