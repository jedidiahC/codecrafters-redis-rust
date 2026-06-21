use crate::{
    resp::Resp,
    store::{RedisStore, StoreElement},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub fn set(key: String, value: String, expiration: Option<Instant>, store: &RedisStore) -> Resp {
    let mut store = store.lock().unwrap();
    let store_value = StoreElement::String { value, expiration };

    store.insert(key, store_value);
    Resp::SimpleString("OK".to_string())
}

