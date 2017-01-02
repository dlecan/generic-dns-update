use std::net::IpAddr;
use regex::Regex;
use xmlrpc::client::Client as XMLRPCClient;
use xmlrpc::protocol::Request as XMLRPCRequest;
use xmlrpc::protocol::Response as XMLRPCResponse;

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
pub enum ZoneVersion {
    LATEST,
    ANOTHER(u16),
}

impl ZoneVersion {
    fn to_number(&self) -> u16 {
        match self {
            &ZoneVersion::LATEST => 0,
            &ZoneVersion::ANOTHER(val) => val,
        }
    }
}

#[derive(Debug)]
pub struct GandiRPC<'a> {
    xmlrpc_server: &'a str,
    apikey: &'a str,
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub ip_addr: String,
    pub record_id: u32,
}

impl<'a> GandiRPC<'a> {
    pub fn new(endpoint: GandiRpcEndpoint, apikey: &'a str) -> GandiRPC {
        GandiRPC {
            xmlrpc_server: endpoint.url(),
            apikey: apikey,
        }
    }

    fn get_gandi_client(&self, rpc_action: &str) -> (XMLRPCClient, XMLRPCRequest) {
        let client = XMLRPCClient::new(self.xmlrpc_server);
        let mut request = XMLRPCRequest::new(rpc_action);
        request = request.argument(&self.apikey.to_string());
        (client, request)
    }

    pub fn domain_info(&self, domain: &str) -> XMLRPCResponse {
        trace!("domain_info - domain: {:?}", domain);

        let (client, mut request) = self.get_gandi_client("domain.info");
        request = request.argument(&domain.to_string());
        request = request.finalize();

        client.remote_call(&request).unwrap()
    }

    pub fn domain_zone_record_list(&self,
                                   record_name: &str,
                                   record_type: &str,
                                   zone_id: &u32,
                                   zone_version: ZoneVersion)
                                   -> Option<Zone> {

        trace!("domain_zone_record_list - record_name: {:?} - record_type: {:?} - zone_id: {:?} \
                - zone_version: {:?}",
               record_name,
               record_type,
               zone_id,
               zone_version);

        let (client, mut request) = self.get_gandi_client("domain.zone.record.list");
        request = request.argument(zone_id);
        request = request.argument(&zone_version.to_number());

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct Record {
            name: String,
            type_: String,
        }

        let record = Record {
            name: record_name.to_string(),
            type_: record_type.to_string(),
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

        let caps = regex.captures(&body);

        let maybe_record_id = caps.map_or(None, |caps| caps.at(1))
            .map(|val| val.parse::<u32>().ok().unwrap());

        maybe_ip_addr.and_then(|ip| {
            maybe_record_id.map(|id| {
                Zone {
                    ip_addr: ip.to_string(),
                    record_id: id,
                }
            })
        })
    }

    pub fn domain_zone_version_new(&self, zone_id: &u32) -> u16 {

        trace!("domain_zone_version_new - zone_id: {:?}", zone_id);

        let (client, mut request) = self.get_gandi_client("domain.zone.version.new");
        request = request.argument(zone_id);
        request = request.finalize();

        let response = client.remote_call(&request).unwrap();

        let regex = Regex::new(r"<int>([0-9]+)</int>").unwrap();

        let caps = regex.captures(&*response.body).unwrap();

        caps.at(1).unwrap().parse::<u16>().ok().unwrap()
    }

    pub fn domain_zone_record_update(&self,
                                     record_name: &str,
                                     record_type: &str,
                                     ip_addr: &IpAddr,
                                     zone_id: &u32,
                                     zone_version: &u16,
                                     new_record_id: &u32) {

        trace!("domain_zone_record_update - record_name: {:?} - record_type: {:?} - ip_addr: \
                {:?} - zone_id: {:?} - zone_version: {:?} - new_record_id: {:?}",
               record_name,
               record_type,
               ip_addr,
               zone_id,
               zone_version,
               new_record_id);

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
            type_: record_type.to_string(),
            value: ip_addr.to_string(),
        };

        request = request.argument(&record);

        request = request.finalize();

        // Horrible hack, because 'type' is a reserved keyword ...
        request.body = request.body.replace("type_", "type");

        client.remote_call(&request); // ignore response
    }

    pub fn domain_zone_version_set(&self, zone_id: &u32, zone_version: &u16) -> bool {

        trace!("domain_zone_version_set - zone_id: {:?} - zone_version: {:?}",
               zone_id,
               zone_version);

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

    pub fn domain_zone_record_add(&self,
                                  record_name: &str,
                                  record_type: &str,
                                  ip_addr: &IpAddr,
                                  zone_id: &u32,
                                  zone_version: &u16) {

        trace!("domain_zone_record_add - record_name: {:?} - record_type: {:?} - ip_addr: {:?} - \
                zone_id: {:?} - zone_version: {:?}",
               record_name,
               record_type,
               ip_addr,
               zone_id,
               zone_version);

        let (client, mut request) = self.get_gandi_client("domain.zone.record.add");
        request = request.argument(zone_id);
        request = request.argument(zone_version);

        #[derive(Debug,RustcEncodable,RustcDecodable)]
        struct Record {
            name: String,
            type_: String,
            value: String,
        }

        let record = Record {
            name: record_name.to_string(),
            type_: record_type.to_string(),
            value: ip_addr.to_string(),
        };

        request = request.argument(&record);

        request = request.finalize();

        // Horrible hack, because 'type' is a reserved keyword ...
        request.body = request.body.replace("type_", "type");

        client.remote_call(&request); // ignore response
    }
}
