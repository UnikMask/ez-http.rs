use ez_http_rs::ThreadPool;
use signal_hook;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

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
