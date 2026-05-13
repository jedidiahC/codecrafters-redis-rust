#![allow(unused_imports)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);

    let mut line = String::new();

    loop {
        line.clear();

        let bytes = buf_reader.read_line(&mut line).await?;
        if bytes == 0 {
            return Ok(());
        }

        if line.trim() == "PING" {
            writer.write_all(b"+PONG\r\n").await?;
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment the code below to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        println!("accepted new connection");
        tokio::spawn(async move { handle_connection(stream).await });
    }
}
