mod get;
mod list;
mod set;

use super::resp::Resp;
use super::store::{RedisStore, StoreElement};
use get::get;
use list::{llen, lpop, lpush, lrange, rpush};
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
    Lpush {
        key: String,
        elements: Vec<String>,
    },
    Lrange {
        key: String,
        start: i64,
        end: i64,
    },
    Llen {
        key: String,
    },
    Lpop {
        key: String,
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
                let mut iter = args.into_iter();
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
                            key,
                            value,
                            expiration: Some(expiration),
                        });
                    }
                }
                Some(Command::Set {
                    key,
                    value,
                    expiration: None,
                })
            }
            "GET" => {
                let mut iter = args.into_iter();
                let key = iter.next()?;
                Some(Command::Get { key })
            }
            "RPUSH" => {
                let mut iter = args.into_iter();
                let key = iter.next()?;
                let elements: Vec<String> = iter.collect();
                Some(Command::Rpush { key, elements })
            }
            "LPUSH" => {
                let mut iter = args.into_iter();
                let key = iter.next()?;
                let elements: Vec<String> = iter.collect();
                Some(Command::Lpush { key, elements })
            }
            "LRANGE" => {
                let mut iter = args.into_iter();
                let key = iter.next()?;
                let start: i64 = iter.next()?.parse().ok()?;
                let end: i64 = iter.next()?.parse().ok()?;
                Some(Command::Lrange { key, start, end })
            }
            "LLEN" => {
                let mut iter = args.into_iter();
                let key = iter.next()?;
                Some(Command::Llen { key })
            }
            "LPOP" => {
                let mut iter = args.into_iter();
                let key = iter.next()?;
                Some(Command::Lpop { key })
            }
            _ => None,
        }
    }

    pub fn run(self, store: RedisStore) -> String {
        let resp: Resp = match self {
            Command::Ping => ping(),
            Command::Echo(message) => echo(message.clone()),
            Command::Set {
                key,
                value,
                expiration,
            } => set(key.clone(), value.clone(), expiration.clone(), &store),
            Command::Get { key } => get(&key, &store),
            Command::Rpush { key, elements } => rpush(key.clone(), &mut elements.clone(), &store),
            Command::Lpush { key, elements } => lpush(key, elements, &store),
            Command::Lrange { key, start, end } => lrange(&key, start, end, &store),
            Command::Llen { key } => llen(key, &store),
            Command::Lpop { key } => lpop(key, &store),
        };

        resp.to_string()
    }
}

pub fn ping() -> Resp {
    Resp::SimpleString("PONG".to_string())
}

pub fn echo(message: String) -> Resp {
    Resp::BulkString(message)
}
