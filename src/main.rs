use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
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

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
