use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
};


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Received a job for worker");
        thread::spawn(move || {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, contents, content_type) = if request_line.starts_with("GET /lib/") || request_line.starts_with("GET /background") {
        // Serve static files (images, etc.)
        let path = request_line.split_whitespace().nth(1).unwrap().trim_start_matches('/');
        match fs::read(path) {
            Ok(bytes) => ("HTTP/1.1 200 OK", bytes, get_content_type(path)),
            Err(_) => ("HTTP/1.1 404 NOT FOUND", Vec::new(), "text/plain"),
        }
    } else {
        let (status_line, filename) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "page1.html"),
            "GET /page1 HTTP/1.1" => ("HTTP/1.1 200 OK", "page1.html"),
            "GET /page2 HTTP/1.1" => ("HTTP/1.1 200 OK", "page2.html"),
            "GET /page3 HTTP/1.1" => ("HTTP/1.1 200 OK", "page3.html"),
            "GET /page4 HTTP/1.1" => ("HTTP/1.1 200 OK", "page4.html"),
            "GET /page5 HTTP/1.1" => ("HTTP/1.1 200 OK", "page5.html"),
            "GET /page6 HTTP/1.1" => ("HTTP/1.1 200 OK", "page6.html"),
            "GET /page7 HTTP/1.1" => ("HTTP/1.1 200 OK", "page7.html"),
            "GET /page8 HTTP/1.1" => ("HTTP/1.1 200 OK", "page8.html"),
            "GET /page9 HTTP/1.1" => ("HTTP/1.1 200 OK", "page9.html"),
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        };
        match fs::read(filename) {
            Ok(bytes) => (status_line, bytes, "text/html"),
            Err(_) => ("HTTP/1.1 404 NOT FOUND", Vec::new(), "text/plain"),
        }
    };

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n");
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
}

fn get_content_type(path: &str) -> &str {
    if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    }
}
