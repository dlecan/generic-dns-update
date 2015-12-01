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
        .arg(Arg::with_name("cnames")
            .help("A space-separated list of the name(s) of the A record(s) to update or create")
            .index(1)
            .required(true)
            .multiple(true))
        .get_matches();

    // Init logging to DEBUG only if user required it
    if matches.is_present("verbose") {
        env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init().unwrap();

    let apikey = matches.value_of("apikey").unwrap();
    debug!("Using apikey: {}", apikey);

    let domain = matches.value_of("domain").unwrap();
    debug!("Using domain: {}", domain);

    if let Some(ref in_v) = matches.values_of("cname") {
        for in_cname in in_v.iter() {
            debug!("cname: {}", in_cname);
        }
    }

    let dry_run = matches.is_present("dry-run");
    debug!("Dry run: {}", dry_run);

}
