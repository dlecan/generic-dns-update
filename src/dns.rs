use config::Config;
use error::Result;
use ip::IpAddr;
use gandi::GandiRPC;
use gandi::GandiRpcEndpoint;
use gandi::ZoneVersion;
use std::str::FromStr;

pub struct DNSProviderFactory;

impl<'a> DNSProviderFactory {
    pub fn build(config: &'a Config) -> Box<DNSProvider + 'a>  {
        Box::new(GandiDNSProvider::new(&config.apikey))
    }
}

#[derive(Debug)]
pub struct Record<'a> {
    pub name: &'a str,
    pub type_: RecordType,
}

impl<'a> Record<'a> {
    pub fn new(record_name: &'a str, ip_addr: &IpAddr) -> Record<'a> {
        let record_type = RecordType::from_ipaddr(&ip_addr);

        Record {
            name: record_name,
            type_: record_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecordType {
    A,
    AAAA,
}

impl RecordType {
    pub fn to_string(&self) -> String {
        match self {
            &RecordType::A => "A".to_string(),
            &RecordType::AAAA => "AAAA".to_string(),
        }
    }

    pub fn from_ipaddr(ip_addr: &IpAddr) -> RecordType {
        match ip_addr {
            &IpAddr::V4(_) => RecordType::A,
            &IpAddr::V6(_) => RecordType::AAAA,
        }
    }
}

pub trait DNSProvider {
    fn init(&mut self, domain: &str) -> Result<()>;
    fn handle_ipv6_addr(&self) -> bool;
    fn is_record_already_declared(&self, record: &Record) -> Result<Option<IpAddr>>;
    fn update_record(&self, record: &Record, ip_addr: &IpAddr) -> Result<()>;
    fn create_record(&self, record: &Record, ip_addr: &IpAddr) -> Result<()>;
}

pub struct GandiDNSProvider<'a> {
    zone_id: u32,
    gandi_rpc: GandiRPC<'a>,
}

impl<'a> GandiDNSProvider<'a> {
    pub fn new(gandi_apikey: &'a str) -> GandiDNSProvider<'a> {

        let gandi_rpc = GandiRPC::new(GandiRpcEndpoint::PROD, gandi_apikey);

        GandiDNSProvider {
            zone_id: Default::default(),
            gandi_rpc: gandi_rpc,
        }
    }
}

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
        true
    }

    fn is_record_already_declared(&self, record: &Record) -> Result<Option<IpAddr>> {

        let zone = &self.gandi_rpc.domain_zone_record_list(&record.name, &record.type_.to_string(), &self.zone_id, ZoneVersion::LATEST);

        Ok(zone.clone().map(|zone| IpAddr::from_str(&zone.ip_addr).unwrap()))
    }

    fn update_record(&self, record: &Record, ip_addr: &IpAddr) -> Result<()> {

        // Create a new zone and get returned version

        let new_zone_version = &self.gandi_rpc.domain_zone_version_new(&self.zone_id);

        debug!("New zone version: {}", new_zone_version);

        let zone = &self.gandi_rpc.domain_zone_record_list(&record.name, &record.type_.to_string(), &self.zone_id, ZoneVersion::ANOTHER(*new_zone_version)).unwrap();

        debug!("New zone: {:?}", zone);

        // Update zone with the new record
        &self.gandi_rpc.domain_zone_record_update(&record.name,
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

    fn create_record(&self, record: &Record, ip_addr: &IpAddr) -> Result<()> {
        // Create a new zone and get returned version

        let new_zone_version = &self.gandi_rpc.domain_zone_version_new(&self.zone_id);

        debug!("New zone version: {}", new_zone_version);

        // Create the new record for the new zone version
        &self.gandi_rpc.domain_zone_record_add(&record.name,
                                                ip_addr,
                                                &self.zone_id,
                                                new_zone_version);

        // Activate the new zone
        debug!("Activate version '{}' of the zone '{}'",
               new_zone_version,
               &self.zone_id);

        self.gandi_rpc.domain_zone_version_set(&self.zone_id, &new_zone_version);
        // TODO: check previous result
        Ok(())
    }
}
