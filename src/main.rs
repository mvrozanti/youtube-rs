extern crate hyper;
extern crate rustty;
extern crate rustc_serialize;

use std::env::{args, current_dir};
use std::io::prelude::Read;
use std::time::Duration;
use std::thread;
use std::process::{Command};

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
        let id = video.find_path(&["snippet", "resourceId", "videoId"]).unwrap().as_string().unwrap();
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

fn print_videos(term: &mut Terminal, videos: &Vec<Video>) {
    let cell_light =   Cell::with_style(Color::Blue, Color::Default, Attr::Default);
    let cell_dark =    Cell::with_style(Color::Yellow, Color::Default, Attr::Default);

    term.clear().unwrap();
    
    for (i, video) in videos.iter().enumerate() {
        term.printline(0, i * 3, &video.title);
        term.printline_with_cell(0, i * 3 + 1, &video.channel, cell_dark);
    }

    change_active(term, 0, &videos);
}

fn gen_url(limit: usize, token: String) -> String {
    let base_url = "https://www.googleapis.com/youtube/v3/";
    let playlist_id = args().nth(1).expect("No playlist id provided").to_string();

    format!("{}playlistItems?key={}&maxResults={}&pageToken={}&playlistId={}&part=snippet",
             base_url, apikey::KEY, limit, token, playlist_id)
}

fn change_active(term: &mut Terminal, current_video: usize, videos: &Vec<Video>) {
    let cell_active =  Cell::with_style(Color::Default, Color::Magenta, Attr::Default); 
    
    if current_video != 0 {
        term.printline(0, current_video * 3 - 3, &videos[current_video - 1].title);
    }
    
    if current_video != videos.len() - 1 {
        term.printline(0, current_video * 3 + 3, &videos[current_video + 1].title);
    }

    term.printline_with_cell(0, current_video * 3, &videos[current_video].title, cell_active);
}

fn main() {
    let mut term = Terminal::new().expect("Couldn't create terminal");
    term.hide_cursor().unwrap();
    
    let limit = term.rows() / 3;
    let mut url = gen_url(limit, "".to_string());
    let mut video_data = get_videos(&url);
    print_videos(&mut term, &video_data.videos);

    let mut current_video = 0;
    change_active(&mut term, current_video, &video_data.videos);
   
    let mut video_arg = "--force-window";
    for arg in args() {
        match arg.as_str() {
            "--audio" => {
                video_arg = "--no-video";
            }
            _ => {}
        }
    }

    thread::spawn(move || {
        Command::new("mpv")
            .arg(&video_arg)
            .arg("--idle")
            .arg("--input-ipc-server=/tmp/mpvsocket")
            .arg("--really-quiet")
            .output()
            .expect("Couldn't play video");
    });

    let send_command = format!("{}/src/send_command.sh", current_dir().unwrap().display());

    'main: loop {
        while let Some(Event::Key(ch)) = term.get_event(Duration::new(0, 0)).unwrap() {
            match ch {
                'q' => {
                    break 'main;
                }
                '\x36' => {
                    url = gen_url(limit, video_data.next_token);
                    video_data = get_videos(&url);
                    
                    print_videos(&mut term, &video_data.videos);
                    current_video = 0;
                }
                '\x35' => {
                    url = gen_url(limit, video_data.prev_token);
                    video_data = get_videos(&url);
                    
                    print_videos(&mut term, &video_data.videos);
                    current_video = 0;
                }
                'A' => {
                    if current_video != 0 {
                        current_video = current_video - 1;
                        change_active(&mut term, current_video, &video_data.videos);
                    }
                }
                'B' => {
                    if current_video != video_data.videos.len() - 1 {
                        current_video = current_video + 1;
                        change_active(&mut term, current_video, &video_data.videos);
                    }
                }
                '\r' => {
                    Command::new(&send_command).arg("stop").spawn().unwrap();

                    for i in current_video..video_data.videos.len() {
                        let video_url = format!("http://www.youtube.com/watch?v={}", 
                                                video_data.videos[i].id);   
                        if i == current_video {
                            Command::new(&send_command)
                                .arg("loadfile")
                                .arg(&video_url)
                                .spawn()
                                .unwrap();
                        } else {
                            Command::new(&send_command)
                                .arg("loadfile")
                                .arg(&video_url)
                                .arg("append")
                                .spawn()
                                .unwrap();
                        }
                    }
                }
                'p' => {
                    Command::new(&send_command)
                        .arg("keypress")
                        .arg("p")
                        .spawn()
                        .unwrap();
                }
                _ => {}
            }
        }

        term.swap_buffers().unwrap(); 
    }
}
