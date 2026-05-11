#![allow(unused_imports)]
use std::{
    io::{BufRead, BufReader, Read, Result, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let line = line?;
        if line == "PING" {
            stream.write_all(b"+PONG\r\n")?;
        }
    }

    Ok(())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment the code below to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
