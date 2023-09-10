use ez_http_rs::ThreadPool;
use serde::{Deserialize, Serialize};
use signal_hook;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    address: String,
    port: String,
    threads: usize,
}

fn main() {
    // Grab config file
    let config_file = fs::read_to_string("./config.ron");
    if let Err(error) = config_file {
        println!("Couldn't find config file! {:?}", error);
        return
    }
    let config = ron::from_str::<Config>(&config_file.unwrap()); 
    if let Err(error) = config.as_ref() {
        println!("Couldn't parse config file! {:?}", error);
        return;
    }
    let config = config.unwrap();

    let full_address = format!("{}:{}", config.address, config.port);
    let listener = TcpListener::bind(full_address.clone());
    if let Err(error) = listener.as_ref() {
        println!("Couldn't bind to {}! {}", full_address, error);
        return;
    }
    let listener = listener.unwrap();
    let pool = ThreadPool::new(config.threads);

    // Set terminal catching commands for graceful shutdown.
    let term = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term)).unwrap();
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

    for stream in listener.incoming() {
        if term.load(Ordering::Relaxed) {
            break;
        }
        pool.execute(|| handle_connection(stream.unwrap()));
    }
    println!("Shutting down...");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let (fp, status) = match buf_reader.lines().next().unwrap().unwrap().as_ref() {
        "GET / HTTP/1.1" => ("html/main.html", "HTTP/1.1 200 OK"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("html/main.html", "HTTP/1.1 200 OK")
        }
        _ => ("html/404.html", "HTTP/1.1 404 NOT FOUND"),
    };
    stream.write_all(write_page(fp, status).as_bytes()).unwrap();
}

fn write_page(fp: &str, status: &str) -> String {
    let contents = fs::read_to_string(fp).unwrap();
    let length = contents.len();
    format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}")
}
