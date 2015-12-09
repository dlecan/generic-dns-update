#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{Arg, App, SubCommand};
use std::env;
use std::io;
use std::io::prelude::*;

const GANDI_URL_PROD: &'static str = "https://rpc.gandi.net/";

trait GandiAPI {
    fn isRecordAlreadyDeclared(&self, record: &str) -> Option<&str>;
    fn updateRecord(&self, record: &str, ipAddr: &str);
    fn createRecord(&self, record: &str, ipAddr: &str);
}

struct GandiAPIImpl;

impl GandiAPI for GandiAPIImpl {
    fn isRecordAlreadyDeclared(&self, record: &str) -> Option<&str> {
        unimplemented!();
    }

    fn updateRecord(&self, record: &str, ipAddr: &str) {
        unimplemented!();
    }

    fn createRecord(&self, record: &str, ipAddr: &str) {
        unimplemented!();
    }
}

fn main() {
    let matches = App::new("gdu")
        .version(&crate_version!()[..])
        .author("Damien Lecan <dev@dlecan.com>")
        .about("Gandi DNS updater, useful to reflect your dynamic IP address to your Gandi DNS zone file.\nIP address is read from stdin.")
        .args_from_usage(
            "-a --apikey=[apikey] 'Your API key provided by Gandi'
            -d --domain=[domain] 'The domain name whose active zonefile will be updated, e.g. \"domain.com\"'
            -n --dry-run 'Dry run, don't really update Gandi zone file'
            [verbose]... -v 'Verbose mode'")
        .arg(Arg::with_name("record_name")
            .help("Name of the A record to update or create (without domain)")
            .index(1)
            .required(true)
            .multiple(false))
        .get_matches();

    // Init logging to DEBUG only if user requires it
    if matches.is_present("verbose") {
        env::set_var("RUST_LOG", "DEBUG");
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

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            println!("Input: {}", input);
        }
        Err(error) => println!("error: {}", error),
    }

    let detectedIpAddr = input.trim();

    let gandi_api = GandiAPIImpl;

    let maybeChecked = gandi_api.isRecordAlreadyDeclared(record_name);

    match maybeChecked {
        Some(ipAddr) => {
            if ipAddr != detectedIpAddr {
                gandi_api.updateRecord(record_name, ipAddr);
            }
        }
        None => gandi_api.createRecord(record_name, detectedIpAddr)
    }

}
