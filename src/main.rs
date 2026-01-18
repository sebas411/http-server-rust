use std::{io::{Read, Write}, net::TcpListener};

use anyhow::Result;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("Http server started. Accepting connections from 127.0.0.1:4221.");
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Incomming connection from {}", stream.peer_addr().unwrap().to_string());
                let mut buffer = [0; 1024];
                stream.read(&mut buffer)?;
                let buffer_string = String::from_utf8(buffer.to_vec())?;
                let split_buffer = buffer_string.split("\r\n").collect::<Vec<_>>();
                let req = split_buffer[0];
                let path = req.split(' ').nth(1).unwrap_or("");
                let status_code;
                if path == "/" {
                    status_code = 200;
                } else {
                    status_code = 404;
                }
                let status_text;
                if status_code == 404 {
                    status_text = "Not Found".to_string();
                } else {
                    status_text = "OK".to_string();
                }
                let mut response = String::new();
                response.push_str(&format!("HTTP/1.1 {} {}\r\n", status_code, status_text));
                response.push_str("\r\n");
                stream.write_all(response.as_bytes()).expect("Unable to write response to stream.");
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
    Ok(())
}
