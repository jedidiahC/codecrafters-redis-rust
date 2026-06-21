use std::fmt;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::ReadHalf;

/**
 * Util module for handling RESP.
 */

/**
 * This module handles the following.
 * Input -> "raw string"
 * 1. First we treat the raw string as an array of bulk strings.
 * 2. We tokenize the bulk string.
 * 3. Parse the tokens into a command.
 * 4. We return the command object to the caller which will execute the command.
 */

pub enum Resp {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<Resp>),
    Null,
}

impl fmt::Display for Resp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resp::SimpleString(s) => write!(f, "+{}\r\n", s),
            Resp::SimpleError(e) => write!(f, "-{}\r\n", e),
            Resp::Integer(i) => write!(f, ":{}\r\n", i),
            Resp::BulkString(b) => write!(f, "${}\r\n{b}\r\n", b.len()),
            Resp::Null => write!(f, "$-1\r\n"),
            Resp::Array(arr) => {
                let arr_strings: Vec<String> = arr.iter().map(|e| e.to_string()).collect();
                write!(f, "*{}\r\n{}", arr.len(), arr_strings.join(""))
            }
        }
    }
}

pub async fn read_bulk_string_array(reader: ReadHalf<'_>) -> Option<Vec<String>> {
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

    // Read RESP bulk string array.
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

    Some(args)
}
