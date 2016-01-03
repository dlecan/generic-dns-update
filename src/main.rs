mod error;

// My IP address providers
mod myip;

// DNS providers
mod dns;

// Configuration
mod config;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate xmlrpc;
extern crate rustc_serialize;
extern crate regex;
extern crate ip;
extern crate hyper;

use clap::{Arg, App};
use std::env;

use config::Config;
use dns::*;
use error::Result;
use myip::*;
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
        .about("Gandi DNS updater, useful to reflect your dynamic IP address to your Gandi DNS zone file.\nIP address is read from stdin.")
        .args_from_usage(
            "-a --apikey=[apikey] 'Your API key provided by Gandi'
            -d --domain=[domain] 'The domain name whose active zonefile will be updated, e.g. \"domain.com\"'
            -n --dry-run 'Dry run, don't really update Gandi zone file'
            -f --force 'Force new zonefile creation even if IP address isn\'t modified'
            -r --record-name=[record_name] 'Name of the A record to update or create (without domain)'
            [verbose]... -v 'Verbose mode'")
        .arg(Arg::with_name("ip_provider")
            .help("IP address provider name to use to get IP address.")
            .short("i")
            .long("ip-provider")
            .takes_value(true)
            .multiple(false))
        .get_matches();

    // Init logging to DEBUG only if user requires it
    match matches.occurrences_of("verbose") {
        0 => env::set_var("RUST_LOG", "INFO"),
        1 => {
            println!("Verbose mode");
            env::set_var("RUST_LOG", "DEBUG");
        }
        2 | _=> {
            println!("More verbose mode");
            env::set_var("RUST_LOG", "TRACE");
        }
    }
    env_logger::init().unwrap();

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

    let ip_provider = IpAddressProvider::from_config(config);

    let my_ip = try!(ip_provider.get_my_ip_addr());

    info!("My IP address: {}", my_ip);

    // Force Gandi DNS provider for now
    let mut dns_provider = dns::GandiDNSProvider::new(&config.apikey);

    try!(dns_provider.init(&config.domain));

    match my_ip {
        ip::IpAddr::V6(_)
            if !dns_provider.handle_ipv6_addr() => panic!("You cannot use IP v6 addresses with the selected DNS provider"),
        _ => (),
    }

    let maybe_checked = try!(dns_provider.is_record_already_declared(&config.record_name));

    match maybe_checked {
        Some(ip_addr) => {
            debug!("Record already declared, with IP address: {}", &ip_addr);

            if !config.force && (&ip_addr == &my_ip) {
                info!("IP address not modified, no record to update");
                Ok(())
            } else {
                info!("Update record '{}' with IP address '{}'", &config.record_name, &my_ip);
                Ok(try!(dns_provider.update_record(&config.record_name, &my_ip)))
            }
        }
        None => Ok(try!(dns_provider.create_record(&config.record_name, &my_ip)))
    }
}
