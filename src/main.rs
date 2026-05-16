#![allow(unused_imports)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::ReadHalf;
use tokio::net::{TcpListener, TcpStream};

trait Resp {
    fn to_resp(&self) -> String;
    fn to_bulk_str(&self) -> String;
}

impl Resp for str {
    fn to_resp(&self) -> String {
        format!("+{}\r\n", self)
    }
    fn to_bulk_str(&self) -> String {
        format!("${}\r\n{}\r\n", self.len(), self)
    }
}

type Store = Arc<Mutex<HashMap<String, Bytes>>>;

async fn read_command_arguments(reader: ReadHalf<'_>) -> Option<(String, Vec<String>)> {
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    line.clear();

    let bytes = buf_reader.read_line(&mut line).await.ok()?;
    if bytes == 0 {
        return None;
    }

    // Check if raw input is RESP array.
    let first = line.trim();

    // If the first byte is not *, not a valid RESP array.
    if !first.starts_with('*') {
        return None;
    }

    let number_of_elements = first[1..].parse().ok()?;
    let mut args: Vec<String> = Vec::new();

    for _ in 0..number_of_elements {
        line.clear();

        // Skip length element.
        let bytes = buf_reader.read_line(&mut line).await.ok()?;
        if bytes == 0 {
            break;
        }

        line.clear();

        // Process content element.
        let bytes = buf_reader.read_line(&mut line).await.ok()?;
        if bytes == 0 {
            break;
        }

        args.push(line.trim().to_string());
    }

    return args
        .split_first()
        .map(|(first, rest)| (first.clone(), rest.to_vec()));
}

// Process command, returning output string.
fn handle_command(command: &String, args: &Vec<String>, store: Store) -> Option<String> {
    if command == "PING" {
        return Some("PONG".to_resp().to_string());
    }

    if command == "ECHO" {
        return Some(args.join(" ").to_bulk_str());
    }

    if command == "SET" {
        let mut store = store.lock().unwrap();
        let key = args[0].clone();
        let value = Bytes::from(args[1..].join(" "));

        store.insert(key, value);
        return Some("OK".to_resp().to_string());
    }

    if command == "GET" {
        let store = store.lock().unwrap();
        let key = args[0].clone();

        if let Some(value) = store.get(&key) {
            return Some(String::from_utf8(value.to_vec()).ok()?.to_bulk_str());
        } else {
            let null_bulk_string = String::from("$-1\r\n");
            return Some(null_bulk_string);
        }
    }

    None
}

async fn handle_connection(mut stream: TcpStream, store: Store) -> anyhow::Result<()> {
    loop {
        let (reader, mut writer) = stream.split();

        if let Some((command, arguments)) = read_command_arguments(reader).await {
            let store = store.clone();
            let output_result = handle_command(&command, &arguments, store);

            if let Some(output) = output_result {
                writer.write_all(output.as_bytes()).await?;
            }
        } else {
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let store: Store = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let store = store.clone();

        println!("accepted new connection");
        tokio::spawn(async move { handle_connection(stream, store).await });
    }
}
