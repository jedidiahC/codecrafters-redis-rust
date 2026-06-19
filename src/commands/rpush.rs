use crate::{
    resp::Resp,
    store::{RedisStore, StoreElement},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub fn rpush(key: String, elements: &mut Vec<String>, store: &RedisStore) -> Resp {
    let mut store = store.lock().unwrap();

    let store_element = store
        .entry(key)
        .or_insert(StoreElement::List { value: Vec::new() });

    if let StoreElement::List { value } = store_element {
        value.append(elements);
        return Resp::Integer(value.len() as i64);
    }
    
    // TODO: Should return error.
    return Resp::Null;
}
