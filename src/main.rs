use std::{io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut response = String::new();
                response.push_str("HTTP/1.1 200 OK\r\n");
                response.push_str("\r\n");
                stream.write_all(response.as_bytes()).expect("Unable to write response to stream.");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
