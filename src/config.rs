pub struct Config<'a> {
    pub apikey: &'a str,
    pub domain: &'a str,
    pub record_name: &'a str,
    pub force: bool,
}
