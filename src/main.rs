#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate xmlrpc;
extern crate rustc_serialize;

extern crate regex;

use clap::{Arg, App};
use std::env;
use std::io;
use std::io::prelude::*;

use regex::Regex;

const GANDI_URL_PROD: &'static str = "https://rpc.gandi.net/xmlrpc/";

trait DNSProvider {
    fn init(&mut self, domain: &str);
    fn is_record_already_declared(&self, record_name: &str) -> Option<String>;
    fn update_record(&self, record_name: &str, ip_addr: &str) -> bool;
    fn create_record(&self, record_name: &str, ip_addr: &str);
}

struct GandiDNSProvider<'a> {
    zone_id: u32,
    gandi_rpc: GandiRPC<'a>,
}

impl<'a> GandiDNSProvider<'a> {

    fn new(gandi_url: &'a str, gandi_apikey: &'a str) -> GandiDNSProvider<'a> {

        let gandi_rpc = GandiRPC {
            xmlrpc_server: gandi_url,
            apikey: gandi_apikey,
        };

        GandiDNSProvider {
            zone_id: 0,
            gandi_rpc: gandi_rpc,
        }
    }
}

impl<'a> DNSProvider for GandiDNSProvider<'a> {

    fn init(&mut self, domain: &str) {

        let response = &self.gandi_rpc.get_domain_info(domain);

        // TODO: fix this ugly code
        // Handle errors
        let zone_id_pos = response.body.find("zone_id").unwrap();
        let (_, body_end) = response.body.split_at(zone_id_pos);
        let first_int_markup = body_end.find("<int>").unwrap();
        let (_, body_end) = body_end.split_at(first_int_markup + "<int>".len());
        let first_end_int_markup = body_end.find("</int>").unwrap();
        let (zone_id, _) = body_end.split_at(first_end_int_markup);
        self.zone_id = zone_id.parse::<u32>().ok().unwrap();

        debug!("Zone id: {}", self.zone_id);
    }

    fn is_record_already_declared(&self, record_name: &str) -> Option<String> {

        let response = &self.gandi_rpc.get_record_list(record_name, &self.zone_id, &0);

        // Extract already configured IP address
        // We are looking for something like that: <value><string>55.32.210.10</string></value>
        let regex = Regex::new(r"<value><string>([0-9.]*)</string></value>").unwrap();

        let caps = regex.captures(&*response.body);

        caps
            .map_or(None, |caps| caps.at(1))
            .map(|val| val.to_string())
    }

    fn update_record(&self, record_name: &str, ip_addr: &str) -> bool {

        // Create a new zone and get returned version

        let new_zone_version = &self.gandi_rpc.create_new_zone(&self.zone_id);

        debug!("New zone version: {}", new_zone_version);

        // Extract new record id

        let new_record_id = &self.gandi_rpc.get_record_id(record_name, &self.zone_id, &new_zone_version);

        debug!("New record id: {}", new_record_id);

        // Update zone with the new record

        &self.gandi_rpc.update_zone_with_record(record_name, ip_addr, &self.zone_id, new_zone_version, new_record_id);

        // Activate the new zone
        debug!("Activate version '{}' of the zone '{}'", new_zone_version, &self.zone_id);

        self.gandi_rpc.update_zone_version(&self.zone_id, &new_zone_version)
    }

    fn create_record(&self, record_name: &str, ip_addr: &str) {
        unimplemented!();
    }
}

struct GandiRPC<'a> {
    xmlrpc_server: &'a str,
    apikey: &'a str,
}

impl<'a> GandiRPC<'a> {

    fn get_gandi_client(&self, rpc_action: &str) -> (xmlrpc::Client, xmlrpc::Request) {
        let client = xmlrpc::Client::new(self.xmlrpc_server);
        let mut request = xmlrpc::Request::new(rpc_action);
        request = request.argument(&self.apikey.to_string());
        (client, request)
    }

    fn get_domain_info(&self, domain: &'a str) -> xmlrpc::Response {
        let (client, mut request) = self.get_gandi_client("domain.info");
        request = request.argument(&domain.to_string());
        request = request.finalize();

        client.remote_call(&request).unwrap()
    }

    fn get_record_list(&self, record_name: &str, zone_id: &u32, zone_version: &u16) -> xmlrpc::Response {

        let (client, mut request) = self.get_gandi_client("domain.zone.record.list");
        request = request.argument(zone_id);
        request = request.argument(zone_version);

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct Record {
            name: String,
            type_: String,
        }

        let record = Record { name: record_name.to_string(), type_: "A".to_string() };

        request = request.argument(&record);

        request = request.finalize();

        // Horrible hack, because 'type' is a reserved keyword ...
        request.body = request.body.replace("type_", "type");

        client.remote_call(&request).unwrap()
    }

    fn create_new_zone(&self, zone_id: &u32) -> u16 {
        let (client, mut request) = self.get_gandi_client("domain.zone.version.new");
        request = request.argument(zone_id);
        request = request.finalize();

        let response = client.remote_call(&request).unwrap();

        let regex = Regex::new(r"<int>([0-9]+)</int>").unwrap();

        let caps = regex.captures(&*response.body).unwrap();

        caps.at(1).unwrap().parse::<u16>().ok().unwrap()
    }

    fn get_record_id(&self, record_name: &str, zone_id: &u32, zone_version: &u16) -> u32 {
        let response = &self.get_record_list(record_name, zone_id, zone_version);

        let regex = Regex::new(r"<int>([0-9]+)</int>").unwrap();

        let caps = regex.captures(&*response.body).unwrap();

        caps.at(1).unwrap().parse::<u32>().ok().unwrap()
    }

    fn update_zone_with_record(&self, record_name: &str, ip_addr: &str, zone_id: &u32, zone_version: &u16, new_record_id: &u32) {
        let (client, mut request) = self.get_gandi_client("domain.zone.record.update");
        request = request.argument(zone_id);
        request = request.argument(zone_version);

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct NewRecordId { id: u32 };
        request = request.argument(&NewRecordId{ id: *new_record_id });

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct Record {
            name: String,
            type_: String,
            value: String,
        }

        let record = Record {
            name: record_name.to_string(),
            type_: "A".to_string(),
            value: ip_addr.to_string(),
        };

        request = request.argument(&record);

        request = request.finalize();

        // Horrible hack, because 'type' is a reserved keyword ...
        request.body = request.body.replace("type_", "type");

        client.remote_call(&request); // ignore response
    }

    fn update_zone_version(&self, zone_id: &u32, zone_version: &u16) -> bool {

        let (client, mut request) = self.get_gandi_client("domain.zone.version.set");
        request = request.argument(zone_id);
        request = request.argument(zone_version);
        request = request.finalize();

        let response = client.remote_call(&request).unwrap();

        let regex = Regex::new(r"<boolean>([0-1]*)</boolean>").unwrap();

        let caps = regex.captures(&*response.body).unwrap();

        let result = caps.at(1).unwrap();

        debug!("Activate version result: {}", result);

        match result {
            "1" => true,
            "0" | _ => false,
        }

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

    let force = matches.is_present("force");
    debug!("Force: {}", force);

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            println!("Input: {}", input);
        }
        Err(error) => println!("error: {}", error),
    }

    let expected_ip_addr = input.trim();

    let mut dns_provider = GandiDNSProvider::new(GANDI_URL_PROD, apikey);

    dns_provider.init(domain);

    let maybe_checked = dns_provider.is_record_already_declared(record_name);

    match maybe_checked {
        Some(ip_addr) => {
            debug!("Record already declared, with IP address: {}", &ip_addr);

            if !force && (&ip_addr == expected_ip_addr) {
                debug!("IP address not modified, no record to update");
            } else {
                debug!("Update record '{}' with IP address '{}'", record_name, &expected_ip_addr);
                let result = dns_provider.update_record(record_name, &expected_ip_addr);
                debug!("End of update process with result: {}", result);
            }
        }
        None => dns_provider.create_record(record_name, expected_ip_addr)
    }

}
