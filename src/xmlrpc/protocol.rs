// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

use rustc_serialize::{Encodable,Decodable};
use xmlrpc::encoding;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub body: String,
}

#[derive(Debug)]
pub struct Response {
    pub body: String,
}

impl Request {
    pub fn new(method: &str) -> Request {
        Request {
            method: method.to_string(),
            body: format!("\
            <?xml version=\"1.0\"?>\
            <methodCall><methodName>{}</methodName>\
                <params>", method),
        }
    }

    pub fn argument<T: Encodable>(mut self, object: &T) -> Request {
        let append_body = format!("<param>{}</param>", encoding::encode(object));
        self.body = self.body + &append_body;
        self
    }

    pub fn finalize(mut self) -> Request {
        self.body = self.body + "</params></methodCall>";
        self
    }

}

impl Response {
    pub fn new(body: &str) -> Response {
        Response {
            body: body.to_string(),
        }
    }

    pub fn result<T: Decodable>(&self) -> Result<Vec<T>, encoding::DecoderError> {
        encoding::decode(&self.body)
    }
}

#[cfg(test)]
mod tests {

    #[derive(RustcDecodable, Debug)]
    struct TestObject {
        key1: String,
        key2: f64,
        key3: bool,
    }

    #[test]
    fn test_encode() {

        let expected = "<?xml version=\"1.0\"?><methodCall><methodName>method_name_value</methodName><params><param><value><string>string_value</string></value></param><param><value><double>4.2</double></value></param><param><value><boolean>1</boolean></value></param></params></methodCall>";

        let mut request = super::Request::new("method_name_value");
        request = request.argument(&"string_value".to_string());
        request = request.argument(&4.2);
        request = request.argument(&true);
        request = request.finalize();
        println!("Encoded body: {:?}", request.body);

        assert_eq!(expected, &*request.body);
    }

    #[test]
    fn test_decode() {
      let response = super::Response { body: "<?xml version=\"1.0\" encoding=\"utf-8\"?>
                                  <methodResponse>
                                  <params>
                                   <param>
                                    <value>
                                     <struct>
                                      <member>
                                       <name>key1</name>
                                       <value>
                                        <string>string_value</string>
                                       </value>
                                      </member>
                                      <member>
                                       <name>key2</name>
                                       <value>
                                        <double>4.2</double>
                                       </value>
                                      </member>
                                      <member>
                                       <name>key3</name>
                                       <value>
                                        <boolean>1</boolean>
                                       </value>
                                      </member>
                                     </struct>
                                    </value>
                                   </param>
                                  </params>
                                  </methodResponse>".into() };

      let result = &response.result::<TestObject>().ok().unwrap()[0];
      println!("Decoded result: {:?}", result);

      assert_eq!("string_value".to_string(), result.key1);
      assert_eq!(4.2, result.key2);
      assert_eq!(true, result.key3);
    }
}
