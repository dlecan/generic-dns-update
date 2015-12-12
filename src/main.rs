#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate xmlrpc;

use clap::{Arg, App};
use std::env;
use std::io;
use std::io::prelude::*;

const GANDI_URL_PROD: &'static str = "https://rpc.gandi.net/xmlrpc/";

trait GandiAPI {
    fn init(&mut self, domain: &str);
    fn is_record_already_declared(&self, record: &str) -> Option<&str>;
    fn update_record(&self, record: &str, ip_addr: &str);
    fn create_record(&self, record: &str, ip_addr: &str);
}

struct GandiAPIImpl<'a> {
  xmlrpc_server: &'a str,
  apikey: &'a str,
  zone_id: Option<i32>,
}

impl<'a> GandiAPIImpl<'a> {

    fn new(gandi_url: &'a str, gandi_apikey: &'a str) -> GandiAPIImpl<'a> {
        GandiAPIImpl {
            xmlrpc_server: gandi_url,
            apikey: gandi_apikey,
            zone_id: None,
        }
    }
}

impl<'a> GandiAPI for GandiAPIImpl<'a> {

    fn init(&mut self, domain: &str) {
        let client = xmlrpc::Client::new(self.xmlrpc_server);
        let mut request = xmlrpc::Request::new("domain.info");
        request = request.argument(&self.apikey.to_string());
        request = request.argument(&domain.to_string());
        request = request.finalize();

        let response = client.remote_call(&request).unwrap();

        // TODO: fix this ugly code
        // Handle errors
        let zone_id_pos = response.body.find("zone_id").unwrap();
        let (_, body_end) = response.body.split_at(zone_id_pos);
        let first_int_markup = body_end.find("<int>").unwrap();
        let (_, body_end) = body_end.split_at(first_int_markup + "<int>".len());
        let first_end_int_markup = body_end.find("</int>").unwrap();
        let (zone_id, _) = body_end.split_at(first_end_int_markup);
        self.zone_id = zone_id.parse::<i32>().ok();

        debug!("Zone id: {}", self.zone_id.unwrap());
    }

    fn is_record_already_declared(&self, record: &str) -> Option<&str> {
        unimplemented!();
    }

    fn update_record(&self, record: &str, ip_addr: &str) {
        unimplemented!();
    }

    fn create_record(&self, record: &str, ip_addr: &str) {
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
    match matches.occurrences_of("verbose") {
        0 => (),
        1 => {
                println!("Verbose mode");
                env::set_var("RUST_LOG", "DEBUG");
            },
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

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            println!("Input: {}", input);
        }
        Err(error) => println!("error: {}", error),
    }

    let detected_ip_addr = input.trim();

    let mut gandi_api = GandiAPIImpl::new(GANDI_URL_PROD, apikey);

    gandi_api.init(domain);

    // let maybeChecked = gandi_api.isRecordAlreadyDeclared(record_name);

    // match maybeChecked {
    //     Some(ipAddr) => {
    //         if ipAddr != detectedIpAddr {
    //             gandi_api.updateRecord(record_name, ipAddr);
    //         }
    //     }
    //     None => gandi_api.createRecord(record_name, detectedIpAddr)
    // }

}
