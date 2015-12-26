use std::io;
use std::io::prelude::*;

pub trait MyIPAddressProvider<'a> {
    fn get_my_ip_addr(&self) -> String;
}

pub struct StdinIpProvider;

impl<'a> MyIPAddressProvider<'a> for StdinIpProvider {
    fn get_my_ip_addr(&self) -> String {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                trace!("Input: {}", input);
            }
            Err(error) => error!("error: {}", error),
        }

        input.trim().to_string()
    }
}
