extern crate hyper;
extern crate rustty;
extern crate rustc_serialize;

use std::io::prelude::Read;
use std::time::Duration;

use hyper::client::{Client, Response};
use rustc_serialize::json::Json;
use rustty::{Terminal, Cell, Color, Attr, Event};
use rustty::ui::Painter;

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

struct VideoData {
    videos: Vec<Video>,
    next_token: String,
    prev_token: String
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

fn get_videos(url: &str) -> VideoData {
    let json = get_json(url);
    let mut videos: Vec<Video> = Vec::new();
    
    for video in json.find("items").unwrap().as_array().unwrap().to_owned() {
        let id = video.find("id").unwrap().as_string().unwrap();
        let title = video.find_path(&["snippet", "title"]).unwrap().as_string().unwrap();
        let channel = video.find_path(&["snippet", "channelTitle"]).unwrap().as_string().unwrap();

        videos.push(Video::new(title.to_string(), channel.to_string(), id.to_string()));
    }
    
    let next_token = match json.find("nextPageToken") {
        Some(token) => token.as_string().unwrap().to_string(),
        None        => "".to_string()
    };
    
    let prev_token = match json.find("prevPageToken") {
        Some(token) => token.as_string().unwrap().to_string(),
        None        => "".to_string()
    };

    VideoData {
        videos: videos, 
        next_token: next_token,
        prev_token: prev_token
    }
}

fn print_videos(term: &mut Terminal, videos: Vec<Video>) {
    let cell_light =   Cell::with_style(Color::Blue, Color::Default, Attr::Default);
    let cell_dark =    Cell::with_style(Color::Yellow, Color::Default, Attr::Default);
    let cell_active =  Cell::with_style(Color::Default, Color::Magenta, Attr::Default); 

    term.clear().unwrap();
    for (i, video) in videos.iter().enumerate() {
        term.printline(0, i * 3, &video.title);
        term.printline_with_cell(0, i * 3 + 1, &video.channel, cell_dark);
    }
}

fn main() {
    let mut term = Terminal::new().expect("Couldn't create terminal");
    term.hide_cursor().unwrap();

    let base_url = "https://www.googleapis.com/youtube/v3/";
    let mut url = format!("{}videos?chart=mostPopular&key={}&part=snippet&maxResults=4&pageToken={}",
                      base_url, apikey::KEY, "");

    let mut video_data = get_videos(&url);
    print_videos(&mut term, video_data.videos);

    'main: loop {
        while let Some(Event::Key(ch)) = term.get_event(Duration::new(0, 0)).unwrap() {
            match ch {
                'q' => {
                    break 'main;
                }
                '\x36' => {
                    url = format!("{}videos?chart=mostPopular&key={}&part=snippet&maxResults=4&pageToken={}",
                        base_url, apikey::KEY, video_data.next_token);
                    
                    video_data = get_videos(&url);
                    print_videos(&mut term, video_data.videos);
                }
                '\x35' => {
                    url = format!("{}videos?chart=mostPopular&key={}&part=snippet&maxResults=4&pageToken={}",
                        base_url, apikey::KEY, video_data.prev_token);
                    
                    video_data = get_videos(&url);
                    print_videos(&mut term, video_data.videos);
                }
                _ => {
                }
            }
        }

        term.swap_buffers().unwrap(); 
    }
}
