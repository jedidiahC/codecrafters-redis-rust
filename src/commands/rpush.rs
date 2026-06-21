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
        .or_insert(StoreElement::List { list: Vec::new() });

    if let StoreElement::List { list } = store_element {
        list.append(elements);
        return Resp::Integer(list.len() as i64)
    }
    
    // TODO: Should return error.
    Resp::Null
}
