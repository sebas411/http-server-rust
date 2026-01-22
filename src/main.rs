use std::{env, fs::File, io::Read};

use anyhow::Result;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, task::JoinSet};

async fn handle_client(mut stream: TcpStream, file_directory: &str) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await?;
    let buffer_string = String::from_utf8(buffer.to_vec())?;
    let mut split_buffer = buffer_string.split("\r\n");
    let req = split_buffer.next().unwrap();
    let headers = split_buffer.take_while(|s| !(*s).is_empty()).collect::<Vec<_>>();
    let path = req.split(' ').nth(1).unwrap_or("");
    let mut content = String::new();
    let mut content_headers = String::new();
    let mut content_type = "text/plain".to_string();
    let status_code;
    match path {
        "/" => {
            status_code = 200;
        },
        "/user-agent" => {
            status_code = 200;
            for header in headers {
                let (k, v) = header.split_once(':').unwrap_or(("", ""));
                if k.trim().to_lowercase() == "user-agent" {
                    content = v.trim().to_string();
                }
            }
        },
        s if s.len() > 6 && &s[..6] == "/echo/" => {
            status_code = 200;
            let echo_text = &s[6..];
            content = echo_text.to_string();
        },
        s if s.len() > 6 && &s[..7] == "/files/" => {
            let filename = &s[7..];
            match File::open(format!("{}/{}", &file_directory, &filename)) {
                Ok(mut file) => {
                    status_code = 200;
                    file.read_to_string(&mut content)?;
                    content_type = "application/octet-stream".to_string();
                },
                Err(_) => {
                    status_code = 404;
                }
            }
        },
        _ => {
            status_code = 404;
        },
    }
    let status_text;
    if status_code == 404 {
        status_text = "Not Found".to_string();
    } else {
        status_text = "OK".to_string();
    }
    if !content.is_empty() {
        content_headers = format!("Content-Type: {}\r\nContent-Length: {}\r\n", content_type, content.len());
    }
    let mut response = String::new();
    response.push_str(&format!("HTTP/1.1 {} {}\r\n{}\r\n{}", status_code, status_text, content_headers, content));
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await.expect("Unable to write response to stream.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let file_directory;
    match args.iter().skip_while(|a| a != &"--directory").skip(1).next() {
        Some(dirname) => {
            file_directory = dirname.clone();
        },
        None => {
            file_directory = "".to_string();
        },
    }
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    println!("Http server started. Accepting connections from 127.0.0.1:4221.");
    let mut handles = JoinSet::new();
    loop {
        let stream = listener.accept().await;
        let file_directory = file_directory.clone();
        match stream {
            Ok((stream, socket_addr)) => {
                println!("Incomming connection from {}", socket_addr.to_string());
                handles.spawn(async move {
                    handle_client(stream, &file_directory).await.unwrap();
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
