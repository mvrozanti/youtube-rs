extern crate hyper;
extern crate rustty;
extern crate rustc_serialize;

use hyper::client::{Client, Response};

fn send_get(url: &str) -> Response {
    let client = Client::new();
    client.get(url).send().expect("Error loading url")
}

fn main() {
    send_get("http://youtube.com");
}
