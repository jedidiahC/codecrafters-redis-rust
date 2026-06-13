#![allow(unused_imports)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

type Store = Arc<Mutex<HashMap<String, StoreValue>>>;

/**
 * We can store
 * a value
 * with various options such as EXP: UTC TIME
 */
struct StoreValue {
    value: String,
    options: HashMap<String, String>,
}

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

    // Read RESP array.
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
// e.g. raw: SET mykey value PX 1000
// command: SET
// args: mykey, value, PX, 1000
// args should be parsed to
// key: mykey
// value: value
// option: {
//   PX: 1000
// }
fn handle_command(command: &String, args: &Vec<String>, store: Store) -> Option<String> {
    println!("COMMAND: {}, args: {}", command, args.join("|"));

    if command == "PING" {
        return Some("PONG".to_resp().to_string());
    }

    if command == "ECHO" {
        return Some(args.join(" ").to_bulk_str());
    }

    if command == "SET" {
        let mut store = store.lock().unwrap();

        let mut args_iter = args.iter().peekable();

        // Parse args for keys and options
        let key = args_iter.next()?.clone(); // 1st arg -> key
        let value = args_iter.next()?.clone(); // 2nd arg -> value

        // Parse 3rd arg onwards as options.

        // Handle PX option.
        // set utc time for expiry.
        // We store a serialiable struct.

        let mut options: HashMap<String, String> = HashMap::new();

        while args_iter.peek() != None {
            let arg = args_iter.next()?;
            if arg == "EX" || arg == "PX" {
                let exp_val_ms: u64 = args_iter.next()?.parse().unwrap();
                let exp_time = match arg.as_str() {
                    "EX" => (SystemTime::now() + Duration::from_secs(exp_val_ms))
                        .duration_since(UNIX_EPOCH),
                    _ => (SystemTime::now() + Duration::from_millis(exp_val_ms))
                        .duration_since(UNIX_EPOCH),
                };

                let exp_opt_value = exp_time.unwrap().as_millis().to_string();

                options.insert("EXP".to_string(), exp_opt_value);
            }
        }

        let store_value = StoreValue { value, options };

        store.insert(key, store_value);
        return Some("OK".to_resp().to_string());
    }

    if command == "GET" {
        let mut store = store.lock().unwrap();
        let key = args[0].clone();

        if let Some(store_value) = store.get(&key) {
            for (key, value) in &store_value.options {
                println!("option: {}, {}", key, value);
            }

            // Check expiry if expired, delete value and return null bulk string $-1\r\n
            if let Some(exp_time) = store_value.options.get(&("EXP".to_string())) {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                let exp_time: u128 = exp_time.parse().unwrap();
                println!("current_time: {}, exp_time: {}", current_time, exp_time);

                if exp_time < current_time {
                    store.remove(&key);
                    let null_bulk_string = String::from("$-1\r\n");
                    return Some(null_bulk_string);
                }
            }

            let value = store_value.value.clone();
            return Some(value.to_bulk_str());
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
