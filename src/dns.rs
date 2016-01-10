use config::Config;
use error::Result;
use ip::IpAddr;
use gandi::*;
use std::str::FromStr;

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
