mod error;

// My IP address providers
mod myip;

// DNS providers
mod dns;

// Configuration
mod config;

mod gandi;

mod xmlrpc;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate rustc_serialize;
extern crate regex;
extern crate hyper;

extern crate time;

extern crate xml;

use clap::{Arg, App};
use config::Config;
use dns::DNSProvider;
use dns::DNSProviderFactory;
use dns::Record;
use env_logger::LogBuilder;
use error::Result;
use log::{LogRecord, LogLevelFilter};
use myip::GetMyIpAddr;
use myip::IpProvider;
use std::process;

fn main() {
    let config = build_config();

    match main_with_errors(&config) {
        Ok(_) => info!("Process ends with success"),
        Err(err) => {
            error!("Process failed with result: {}", err);
            process::exit(-1);
        }
    }
}

fn build_config() -> Config {
    let matches = App::new("gdu")
        .version(&crate_version!()[..])
        .author("Damien Lecan <dev@dlecan.com>")
        .about("Generic DNS update, useful to update your dynamic IP address into your DNS provider zone file, e.g. Gandi or Go Daddy.\n\
            GDU detects if your ip address is IPv4 or v6 and and will create a record with type 'A' or 'AAAA' accordingly.\n\n\
            IP address can be read from several HTTP providers or from stdin.\n\
            Only Gandi DNS provider is implemented in this version.")
        .args_from_usage(
            "-a --apikey=<apikey> 'Your API key provided by Gandi'
            -d --domain=<domain> 'The domain name whose active zonefile will be updated, e.g. \"domain.com\"'
            -n --dry-run 'Dry run, don't really update Gandi zone file'
            -f --force 'Force new zonefile creation even if IP address isn\'t modified'
            -r --record-name=<record_name> 'Name of the A or AAAA record to update or create (without domain)'
            [verbose]... -v 'Verbose mode'")
        .arg(Arg::with_name("ip_provider")
            .help("IP address provider to use to get your own IP address.\n                                       \
                Available values for <ip-provider>:\n                                        \
                 opendns       : OpenDNS\n                                        \
                 -             : read IP address from stdin\n                                        \
                 sfrlaboxfibre : French 'SFR Labox Fibre' subscribers")
            .short("i")
            .long("ip-provider")
            .takes_value(true)
            .multiple(false)
            .required(true))
        .get_matches();

    // Init logger
    let format = |record: &LogRecord| {
        let t = time::now();
        format!("{},{:03} - {} - {}",
            time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
            t.tm_nsec / 1000_000,
            record.level(),
            record.args()
        )
    };

    // Init logging to DEBUG only if user requires it
    let log_level_filter = match matches.occurrences_of("verbose") {
        0 => LogLevelFilter::Info,
        1 => {
            println!("Verbose mode");
            LogLevelFilter::Debug
        }
        2 | _=> {
            println!("More verbose mode");
            LogLevelFilter::Trace
        }
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, log_level_filter);
    builder.init().unwrap();

    // Read parameters
    let apikey = matches.value_of("apikey").unwrap();
    debug!("Using apikey: {}", apikey);

    let domain = matches.value_of("domain").unwrap();
    debug!("Using domain: {}", domain);

    let record_name = matches.value_of("record_name").unwrap();
    debug!("Using record name: {}", record_name);

    let dry_run = matches.is_present("dry-run");
    debug!("Dry run: {}", dry_run);

    let force = matches.is_present("force");
    debug!("Force: {}", force);

    let ip_provider = value_t_or_exit!(matches.value_of("ip_provider"), IpProvider);
    debug!("IP address provider: {:?}", ip_provider);

    Config {
        apikey: apikey.to_owned(),
        domain: domain.to_owned(),
        record_name: record_name.to_owned(),
        force: force,
        ip_provider: ip_provider,
    }
}

fn main_with_errors(config: &Config) -> Result<()> {

    let my_ip = try!(config.ip_provider.get_my_ip_addr());

    info!("My IP address: {}", my_ip);

    let mut dns_provider = dns::DNSProviderFactory::build(config);

    match my_ip {
        std::net::IpAddr::V6(_)
            if !dns_provider.handle_ipv6_addr() => panic!("You cannot use IP v6 addresses with the selected DNS provider"),
        _ => (),
    }

    let record = Record::new(&config.record_name, &my_ip);

    try!(dns_provider.init(&config.domain));

    let maybe_checked = try!(dns_provider.is_record_already_declared(&record));

    match maybe_checked {
        Some(ip_addr) => {
            debug!("Record already declared, with IP address: {}", &ip_addr);

            if !config.force && (&ip_addr == &my_ip) {
                info!("IP address not modified, no record to update");
                Ok(())
            } else {
                info!("Update record '{:?}' with IP address '{}'", &record, &my_ip);
                Ok(try!(dns_provider.update_record(&record, &my_ip)))
            }
        }
        None => Ok(try!(dns_provider.create_record(&record, &my_ip)))
    }
}
