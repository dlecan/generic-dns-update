use myip::IpProvider;

pub struct Config {
    pub apikey: String,
    pub domain: String,
    pub record_name: String,
    pub force: bool,
    pub ip_provider: IpProvider,
}
