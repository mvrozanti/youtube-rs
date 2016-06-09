extern crate hyper;
extern crate rustty;
extern crate rustc_serialize;

use std::io::prelude::Read;

use hyper::client::{Client, Response};
use rustc_serialize::json::Json;

mod apikey;

struct Video {
    title: String,
    channel: String,
    id: String
}

impl Video {
    fn new(title: String, channel: String, id: String) -> Video {
        Video {title: title, channel: channel, id: id}
    }
}

fn send_get(url: &str) -> Response {
    let client = Client::new();
    client.get(url).send().expect("Error loading url")
}

fn get_json(url: &str) -> Json {
    let mut body = String::new();
    send_get(url).read_to_string(&mut body).unwrap();
    return Json::from_str(&body).expect("Invald json")
}


fn get_videos(url: &str) -> Vec<Video> {
    let mut videos: Vec<Video> = Vec::new();
    for video in get_json(url).find("items").unwrap().as_array().unwrap().to_owned() {
        let id = video.find("id").unwrap().as_string().unwrap();
        let title = video.find_path(&["snippet", "title"]).unwrap().as_string().unwrap();
        let channel = video.find_path(&["snippet", "channelTitle"]).unwrap().as_string().unwrap();

        videos.push(Video::new(title.to_string(), channel.to_string(), id.to_string()));
    }
    return videos
}

fn main() {
    let base_url = "https://www.googleapis.com/youtube/v3/videos" ;
    let url = format!("{}?chart=mostPopular&key={}&part=snippet&maxResults=4",
                      base_url, apikey::KEY);

    let videos = get_videos(&url);
    println!("{}", videos[1].title);
}
