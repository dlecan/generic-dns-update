// My IP address providers
mod myip;

// DNS providers
mod dns;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate xmlrpc;
extern crate rustc_serialize;

extern crate regex;

extern crate ip;

use clap::{Arg, App};
use std::env;

use dns::*;

use myip::*;

fn main() {
    let matches = App::new("gdu")
        .version(&crate_version!()[..])
        .author("Damien Lecan <dev@dlecan.com>")
        .about("Gandi DNS updater, useful to reflect your dynamic IP address to your Gandi DNS zone file.\nIP address is read from stdin.")
        .args_from_usage(
            "-a --apikey=[apikey] 'Your API key provided by Gandi'
            -d --domain=[domain] 'The domain name whose active zonefile will be updated, e.g. \"domain.com\"'
            -n --dry-run 'Dry run, don't really update Gandi zone file'
            -f --force 'Force new zonefile creation even if IP address isn\'t modified'
            [verbose]... -v 'Verbose mode'")
        .arg(Arg::with_name("record_name")
            .help("Name of the A record to update or create (without domain)")
            .index(1)
            .required(true)
            .multiple(false))
        .get_matches();

    // Init logging to DEBUG only if user requires it
    match matches.occurrences_of("verbose") {
        0 => (),
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

    // Force Stdin IP provider for now
    let ip_provider = StdinIpProvider;
    let expected_ip_addr = ip_provider.get_my_ip_addr().unwrap();

    // Force Gandi DNS provider for now
    let mut dns_provider = dns::GandiDNSProvider::new(apikey);

    dns_provider.init(domain);

    let maybe_checked = dns_provider.is_record_already_declared(record_name);

    match maybe_checked {
        Some(ip_addr) => {
            debug!("Record already declared, with IP address: {}", &ip_addr);

            if !force && (&ip_addr == &expected_ip_addr) {
                debug!("IP address not modified, no record to update");
            } else {
                debug!("Update record '{}' with IP address '{}'", record_name, &expected_ip_addr);
                let result = dns_provider.update_record(record_name, &expected_ip_addr);
                debug!("End of update process with result: {}", result);
            }
        }
        None => dns_provider.create_record(record_name, &expected_ip_addr)
    }

}
