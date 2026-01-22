use anyhow::Result;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, task::JoinSet};

async fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await?;
    let buffer_string = String::from_utf8(buffer.to_vec())?;
    let mut split_buffer = buffer_string.split("\r\n");
    let req = split_buffer.next().unwrap();
    let headers = split_buffer.take_while(|s| !(*s).is_empty()).collect::<Vec<_>>();
    let path = req.split(' ').nth(1).unwrap_or("");
    let mut content = String::new();
    let mut content_headers = String::new();
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
        content_headers = format!("Content-Type: text/plain\r\nContent-Length: {}\r\n", content.len());
    }
    let mut response = String::new();
    response.push_str(&format!("HTTP/1.1 {} {}\r\n{}\r\n{}", status_code, status_text, content_headers, content));
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await.expect("Unable to write response to stream.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    println!("Http server started. Accepting connections from 127.0.0.1:4221.");
    let mut handles = JoinSet::new();
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, socket_addr)) => {
                println!("Incomming connection from {}", socket_addr.to_string());
                handles.spawn(async move {
                    handle_client(stream).await.unwrap();
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
