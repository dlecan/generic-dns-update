use config::Config;
use error::Result;
use ip::IpAddr;
use regex::Regex;
use std::str::FromStr;
use xmlrpc::Client as XMLRPCClient;
use xmlrpc::Request as XMLRPCRequest;
use xmlrpc::Response as XMLRPCResponse;

pub struct DNSProviderFactory;

impl<'a> DNSProviderFactory {
    pub fn build(config: &'a Config) -> Box<DNSProvider + 'a>  {
        Box::new(GandiDNSProvider::new(&config.apikey))
    }
}

pub trait DNSProvider {
    fn init(&mut self, domain: &str) -> Result<()>;
    fn handle_ipv6_addr(&self) -> bool;
    fn is_record_already_declared(&self, record_name: &str) -> Result<Option<IpAddr>>;
    fn update_record(&self, record_name: &str, ip_addr: &IpAddr) -> Result<()>;
    fn create_record(&self, record_name: &str, ip_addr: &IpAddr) -> Result<()>;
}

pub struct GandiDNSProvider<'a> {
    zone_id: u32,
    gandi_rpc: GandiRPC<'a>,
}

impl<'a> GandiDNSProvider<'a> {
    pub fn new(gandi_apikey: &'a str) -> GandiDNSProvider<'a> {

        let gandi_rpc = GandiRPC {
            xmlrpc_server: GandiRpcEndpoint::PROD.url(),
            apikey: gandi_apikey,
        };

        GandiDNSProvider {
            zone_id: 0,
            gandi_rpc: gandi_rpc,
        }
    }
}

static ZONE_VERSION_LATEST: u16 = 0;

impl<'a> DNSProvider for GandiDNSProvider<'a> {
    fn init(&mut self, domain: &str) -> Result<()> {

        let response = &self.gandi_rpc.domain_info(domain);

        // TODO: fix this ugly code
        // Handle errors
        let zone_id_pos = response.body.find("zone_id").unwrap();
        let (_, body_end) = response.body.split_at(zone_id_pos);
        let first_int_markup = body_end.find("<int>").unwrap();
        let (_, body_end) = body_end.split_at(first_int_markup + "<int>".len());
        let first_end_int_markup = body_end.find("</int>").unwrap();
        let (zone_id, _) = body_end.split_at(first_end_int_markup);

        self.zone_id = try!(zone_id.parse::<u32>());

        debug!("Zone id: {}", self.zone_id);
        Ok(())
    }

    fn handle_ipv6_addr(&self) -> bool {
        // IPv6 addresses are not handled yet
        false
    }

    fn is_record_already_declared(&self, record_name: &str) -> Result<Option<IpAddr>> {

        let zone = &self.gandi_rpc.domain_zone_record_list(record_name, &self.zone_id, &ZONE_VERSION_LATEST);

        Ok(zone.clone().map(|zone| IpAddr::from_str(&zone.ip_addr).unwrap()))
    }

    fn update_record(&self, record_name: &str, ip_addr: &IpAddr) -> Result<()> {

        // Create a new zone and get returned version

        let new_zone_version = &self.gandi_rpc.domain_zone_version_new(&self.zone_id);

        debug!("New zone version: {}", new_zone_version);

        let zone = &self.gandi_rpc.domain_zone_record_list(record_name, &self.zone_id, &new_zone_version).unwrap();

        debug!("New zone: {:?}", zone);

        // Update zone with the new record
        &self.gandi_rpc.domain_zone_record_update(record_name,
                                                ip_addr,
                                                &self.zone_id,
                                                new_zone_version,
                                                &zone.record_id);

        // Activate the new zone
        debug!("Activate version '{}' of the zone '{}'",
               new_zone_version,
               &self.zone_id);

        self.gandi_rpc.domain_zone_version_set(&self.zone_id, &new_zone_version);
        // TODO: check previous result
        Ok(())
    }

    fn create_record(&self, record_name: &str, ip_addr: &IpAddr) -> Result<()> {
        unimplemented!();
    }
}

#[derive(Debug)]
pub enum GandiRpcEndpoint {
    PROD,
    STAGING,
}

impl GandiRpcEndpoint {
    pub fn url(&self) -> &'static str {
        match self {
            &GandiRpcEndpoint::PROD => "https://rpc.gandi.net/xmlrpc/",
            &GandiRpcEndpoint::STAGING => unimplemented!(),
        }
    }
}

#[derive(Debug)]
struct GandiRPC<'a> {
    xmlrpc_server: &'a str,
    apikey: &'a str,
}

#[derive(Debug, Clone)]
struct Zone {
    ip_addr: String,
    record_id: u32,
}

impl<'a> GandiRPC<'a> {
    fn get_gandi_client(&self, rpc_action: &str) -> (XMLRPCClient, XMLRPCRequest) {
        let client = XMLRPCClient::new(self.xmlrpc_server);
        let mut request = XMLRPCRequest::new(rpc_action);
        request = request.argument(&self.apikey.to_string());
        (client, request)
    }

    fn domain_info(&self, domain: &str) -> XMLRPCResponse {
        let (client, mut request) = self.get_gandi_client("domain.info");
        request = request.argument(&domain.to_string());
        request = request.finalize();

        client.remote_call(&request).unwrap()
    }

    fn domain_zone_record_list(&self,
                       record_name: &str,
                       zone_id: &u32,
                       zone_version: &u16)
                       -> Option<Zone> {

        let (client, mut request) = self.get_gandi_client("domain.zone.record.list");
        request = request.argument(zone_id);
        request = request.argument(zone_version);

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct Record {
            name: String,
            type_: String,
        }

        let record = Record {
            name: record_name.to_string(),
            type_: "A".to_string(),
        };

        request = request.argument(&record);

        request = request.finalize();

        // Horrible hack, because 'type' is a reserved keyword ...
        request.body = request.body.replace("type_", "type");

        let body = client.remote_call(&request).unwrap().body;

        // IP address
        let regex = Regex::new(r"<value><string>([0-9.]*)</string></value>").unwrap();

        let caps = regex.captures(&body);

        let maybe_ip_addr = caps.map_or(None, |caps| caps.at(1));

        // record_id
        let regex = Regex::new(r"<int>([0-9]+)</int>").unwrap();

        let caps = regex.captures(&body).unwrap();

        let maybe_record_id = caps.at(1).map(|val| val.parse::<u32>().ok().unwrap());

        maybe_ip_addr.and_then(|ip| {
            maybe_record_id.map(|id| {
                Zone {
                    ip_addr: ip.to_string(),
                    record_id: id,
                }
            })
        })
    }

    fn domain_zone_version_new(&self, zone_id: &u32) -> u16 {
        let (client, mut request) = self.get_gandi_client("domain.zone.version.new");
        request = request.argument(zone_id);
        request = request.finalize();

        let response = client.remote_call(&request).unwrap();

        let regex = Regex::new(r"<int>([0-9]+)</int>").unwrap();

        let caps = regex.captures(&*response.body).unwrap();

        caps.at(1).unwrap().parse::<u16>().ok().unwrap()
    }

    fn domain_zone_record_update(&self,
                               record_name: &str,
                               ip_addr: &IpAddr,
                               zone_id: &u32,
                               zone_version: &u16,
                               new_record_id: &u32) {
        let (client, mut request) = self.get_gandi_client("domain.zone.record.update");
        request = request.argument(zone_id);
        request = request.argument(zone_version);

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct NewRecordId {
            id: u32,
        };
        request = request.argument(&NewRecordId { id: *new_record_id });

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

    fn domain_zone_version_set(&self, zone_id: &u32, zone_version: &u16) -> bool {

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
