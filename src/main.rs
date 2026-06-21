#![allow(unused_imports)]
mod commands;
mod resp;
mod store;

use store::RedisStore;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::ReadHalf;
use tokio::net::{TcpListener, TcpStream};

use crate::commands::Command;

async fn handle_connection(mut stream: TcpStream, store: store::RedisStore) -> anyhow::Result<()> {
    loop {
        let (reader, mut writer) = stream.split();

        if let Some(parts) = resp::read_bulk_string_array(reader).await {
            let store = store.clone();
            let command = Command::parse(parts);

            if let Some(command) = command {
                let output = command.run(store);
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
    let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let store = store.clone();

        println!("accepted new connection");
        tokio::spawn(async move { handle_connection(stream, store).await });
    }
}
