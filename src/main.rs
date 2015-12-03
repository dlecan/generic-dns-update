#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{Arg, App, SubCommand};
use std::env;

const GANDI_URL: &'static str = "https://rpc.gandi.net/";

fn main() {
    let matches = App::new("gdu")
        .version(&crate_version!()[..])
        .author("Damien Lecan <dev@dlecan.com>")
        .about("Gandi DNS updater, useful to reflect your dynamic IP address to your Gandi DNS zone file")
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

}
