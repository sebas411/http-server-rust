use std::{env, fs::File, io::{Read, Write}};

use anyhow::Result;
use flate2::{Compression, write::GzEncoder};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, task::JoinSet};

async fn handle_client(mut stream: TcpStream, file_directory: &str) -> Result<()> {
    let mut buffer = [0; 1024];
    let n_read = stream.read(&mut buffer).await?;
    let buffer_string = String::from_utf8(buffer[..n_read].to_vec())?;
    let (head, body) = buffer_string.split_once("\r\n\r\n").unwrap();
    let mut head_split = head.split("\r\n");

    let req = head_split.next().unwrap();
    let headers = head_split.take_while(|s| !(*s).is_empty()).collect::<Vec<_>>();

    let mut req_split = req.split(' ');
    let method = req_split.next().unwrap_or("");
    let path = req_split.next().unwrap_or("");

    let mut content = vec![];
    let mut content_headers = String::new();
    let mut content_type = "text/plain".to_string();
    let status_code;

    // Endpoints
    match path {
        "/" => {
            status_code = 200;
        },
        "/user-agent" => {
            status_code = 200;
            for header in headers {
                let (k, v) = header.split_once(':').unwrap_or(("", ""));
                if k.trim().to_lowercase() == "user-agent" {
                    content = v.trim().as_bytes().to_vec();
                    break;
                }
            }
        },
        s if s.len() > 6 && &s[..6] == "/echo/" => {
            status_code = 200;
            let echo_text = &s[6..];
            let mut encoded = false;
            for header in headers {
                let (k, v) = header.split_once(':').unwrap_or(("", ""));
                if k.trim().to_lowercase() == "accept-encoding" && v.split(',').any(|scheme| scheme.trim() == "gzip") {
                    content_headers.push_str("Content-Encoding: gzip\r\n");
                    encoded = true;
                    break;
                }
            }
            if encoded {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(echo_text.as_bytes()).unwrap();
                content = encoder.finish().unwrap();
            } else {
                content = echo_text.as_bytes().to_vec();
            }
        },
        s if s.len() > 6 && &s[..7] == "/files/" => {
            let filename = &s[7..];
            if method == "GET" {
                match File::open(format!("{}/{}", &file_directory, &filename)) {
                    Ok(mut file) => {
                        status_code = 200;
                        file.read_to_end(&mut content)?;
                        content_type = "application/octet-stream".to_string();
                    },
                    Err(_) => {
                        status_code = 404;
                    }
                }
            } else {
                status_code = 201;
                if let Ok(mut file) = File::create(format!("{}/{}", &file_directory, &filename)) {
                    file.write_all(body.as_bytes())?;
                }
            }
        },
        _ => {
            status_code = 404;
        },
    }

    let status_text;
    match status_code {
        404 => status_text = "Not Found".to_string(),
        201 => status_text = "Created".to_string(),
        200 => status_text = "OK".to_string(),
        _ => status_text = "".to_string(),
    }
    if !content.is_empty() {
        content_headers.push_str(&format!("Content-Type: {}\r\nContent-Length: {}\r\n", content_type, content.len()));
    }

    // Build and send response for client
    let mut response = vec![];
    response.extend(format!("HTTP/1.1 {} {}\r\n{}\r\n", status_code, status_text, content_headers).as_bytes());
    response.extend(content);
    response.extend(b"\r\n");
    stream.write_all(&response).await.expect("Unable to write response to stream.");
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
