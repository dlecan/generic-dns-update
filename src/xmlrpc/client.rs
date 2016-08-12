// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

use hyper;
use hyper::header::Headers;
use std::string;
use std::io::Read;
use xmlrpc::protocol::{Request, Response};

pub struct Client {
    url: string::String,
}

impl Client {
    pub fn new(s: &str) -> Client {
        Client { url: s.to_string() }
    }

    pub fn remote_call(&self, request: &Request) -> Option<Response> {
        let http_client = hyper::Client::new();
        let mut headers = Headers::new();
        headers.set_raw("Content-Type", vec![b"text/xml".to_vec()]);
        headers.set_raw("User-Agent", vec![b"rust-xmlrpc".to_vec()]);

        debug!("Send XMLRPC request to: {}", &self.url);
        trace!("XMLRPC body: {}", &request.body);

        let response = http_client.post(&self.url)
            .headers(headers)
            .body(&request.body) // FIXME: use to_xml() somehow?
            .send();

        let mut body = String::new();
        response.ok().unwrap().read_to_string(&mut body).ok().expect("could not read response");

        trace!("Reponse body: {}", &body);

        Some(Response::new(&body)) // FIXME: change to a Result<> type
    }
}
