extern crate hyper;
extern crate rustty;
extern crate rustc_serialize;

use std::io::prelude::Read;

use hyper::client::{Client, Response};
use rustc_serialize::json::Json;

mod apikey;

fn send_get(url: &str) -> Response {
    let client = Client::new();
    client.get(url).send().expect("Error loading url")
}

fn get_json(url: &str) -> Json {
    let mut body = String::new();
    send_get(url).read_to_string(&mut body).unwrap();
    return Json::from_str(&body).expect("Invald json")
}

fn main() {
    let base_url = "https://www.googleapis.com/youtube/v3/videos" ;
    let url = format!("{}?chart=mostPopular&key={}&part=snippet&maxResults=4",
                      base_url, apikey::KEY);
    println!("{}", get_json(&url).find_path(&["kind"]).unwrap().to_string());
}
