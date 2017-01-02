// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

// Derived from Rust JSON library
// https://github.com/rust-lang/rustc-serialize

//#![crate_name = "xmlrpc"]
//#![comment = "Rust XML-RPC library"]
//#![license = "Apache/MIT"]
//#![crate_type = "rlib"]
//#![crate_type = "dylib"]

#![forbid(non_camel_case_types)]
#![allow(missing_docs)]

//! XML-RPC library, including both serialization and remote procedure calling
//!
//! # What is XML-RPC?
//!
//! Documentation to be written ... (follow example in json.rs)
//!
//! Basic documentation found on Wikipedia
//! http://en.wikipedia.org/wiki/XML-RPC
//!
//! Full specification of the XML-RPC protocol is found here:
//! http://xmlrpc.scripting.com/spec.html
//!
//! Additional errata and hints can be found here:
//! http://effbot.org/zone/xmlrpc-errata.htm
//!

extern crate rustc_serialize;
extern crate xml;
extern crate hyper;

// pub use encoding::{encode,decode,Encoder,Decoder,Xml};
// pub use client::{Client};
// pub use protocol::{Request,Response};
pub mod encoding;
pub mod client;
pub mod protocol;
#[cfg(test)]
mod tests {}
