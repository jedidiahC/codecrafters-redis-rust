mod get;
mod rpush;
mod set;

use super::resp::Resp;
use super::store::{RedisStore, StoreElement};
use get::get;
use rpush::rpush;
use set::set;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::ReadHalf;
use tokio::net::{TcpListener, TcpStream};

pub enum Command {
    Ping,
    Echo(String),
    Set {
        key: String,
        value: String,
        expiration: Option<Instant>,
    },
    Get {
        key: String,
    },
    Rpush {
        key: String,
        elements: Vec<String>,
    },
}

impl Command {
    pub fn parse(parts: Vec<String>) -> Option<Self> {
        let mut iter = parts.into_iter();
        let command_name = iter.next()?;
        let args: Vec<String> = iter.collect();

        match command_name.to_uppercase().as_str() {
            // PING
            "PING" => Some(Command::Ping),
            // ECHO message
            "ECHO" => {
                let mut iter = args.iter();
                let message = iter.next()?;
                Some(Command::Echo(message.clone()))
            }
            // SET key value [NX | XX | IFEQ ifeq-value | IFNE ifne-value |
            // IFDEQ ifdeq-digest | IFDNE ifdne-digest] [GET] [EX seconds |
            // PX milliseconds | EXAT unix-time-seconds |
            // PXAT unix-time-milliseconds | KEEPTTL]
            "SET" => {
                let mut iter = args.iter();
                let key = iter.next()?;
                let value = iter.next()?;

                let option = iter.next();
                if let Some(option) = option {
                    let option_value = iter.next()?;

                    if option == "EX" || option == "PX" {
                        let exp_val: u64 = option_value.parse().ok()?;
                        let expiration = match option.as_str() {
                            "EX" => Instant::now() + Duration::from_secs(exp_val),
                            _ => Instant::now() + Duration::from_millis(exp_val),
                        };
                        return Some(Command::Set {
                            key: key.clone(),
                            value: value.clone(),
                            expiration: Some(expiration),
                        });
                    }
                }
                Some(Command::Set {
                    key: key.clone(),
                    value: value.clone(),
                    expiration: None,
                })
            }
            "GET" => {
                let mut iter = args.iter();
                let key = iter.next()?;
                Some(Command::Get { key: key.clone() })
            }
            "RPUSH" => {
                let mut iter = args.iter();
                let key = iter.next()?;
                let element = iter.next()?;
                Some(Command::Rpush {
                    key: key.clone(),
                    elements: [element.clone()].to_vec(),
                })
            }
            _ => None,
        }
    }

    pub fn run(&self, store: RedisStore) -> String {
        let resp: Resp = match self {
            Command::Ping => ping(),
            Command::Echo(message) => echo(message.clone()),
            Command::Set {
                key,
                value,
                expiration,
            } => set(key.clone(), value.clone(), expiration.clone(), &store),
            Command::Get { key } => get(key, &store),
            Command::Rpush { key, elements } => rpush(key.clone(), &mut elements.clone(), &store),
        };

        resp.to_string()
    }
}

pub fn ping() -> Resp {
    return Resp::SimpleString("PONG".to_string());
}

pub fn echo(message: String) -> Resp {
    return Resp::BulkString(message);
}
