use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    process,
};
use web_server::{Router, ThreadPool};

const BIND_URL: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(BIND_URL).unwrap_or_else(|err| {
        eprintln!("Couldn't bind to requested port.");
        eprintln!("Error occurred: {err}");
        process::exit(1)
    });
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        })
    }
}

/// Handles a connection from a stream
///
/// Serves as a basic router
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let mut request_line: Option<String> = None;

    match buf_reader.lines().next() {
        Some(val) => request_line = val.ok(),
        None => (),
    }

    let router = Router::new(request_line);

    router.respond(&mut stream)
}
