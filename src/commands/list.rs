use crate::{
    resp::Resp,
    store::{RedisStore, StoreElement},
};

use std::{
    cmp::{max, min},
    collections::HashMap,
    time::{Duration, Instant},
};

pub fn lpush(key: String, elements: Vec<String>, store: &RedisStore) -> Resp {
    let mut store = store.lock().unwrap();

    let store_element = store
        .entry(key)
        .or_insert(StoreElement::List { list: Vec::new() });

    if let StoreElement::List { list } = store_element {
        list.splice(0..0, elements.into_iter().rev());
        return Resp::Integer(list.len() as i64);
    }

    // TODO: Should return error.
    Resp::Null
}

fn normalize_index(index: i64, len: usize) -> usize {
    usize::try_from({
        if index < 0 {
            max(0, len as i64 + index)
        } else {
            min(len as i64 - 1, index) // If end > length of list, treat end as last element.
        }
    })
    .unwrap()
}

pub fn lrange(key: &String, start: i64, end: i64, store: &RedisStore) -> Resp {
    let store = store.lock().unwrap();
    let element = store.get(key);

    if let Some(element) = element {
        if let StoreElement::List { list } = element
            && list.len() > 0
            && normalize_index(start, list.len()) < list.len()
        {
            let start = normalize_index(start, list.len());
            let end = normalize_index(end, list.len());

            let slice = &list[start..=usize::try_from(end).unwrap()];

            return Resp::Array(
                slice
                    .into_iter()
                    .map(|s| Resp::BulkString(s.clone()))
                    .collect(),
            );
        }
    }

    Resp::Array(Vec::new())
}

pub fn rpush(key: String, elements: &mut Vec<String>, store: &RedisStore) -> Resp {
    let mut store = store.lock().unwrap();

    let store_element = store
        .entry(key)
        .or_insert(StoreElement::List { list: Vec::new() });

    if let StoreElement::List { list } = store_element {
        list.append(elements);
        return Resp::Integer(list.len() as i64);
    }

    // TODO: Should return error.
    Resp::Null
}

pub fn llen(key: String, store: &RedisStore) -> Resp {
    let store = store.lock().unwrap();
    let element = store.get(&key);

    if let Some(element) = element {
        if let StoreElement::List { list } = element {
            return Resp::Integer(list.len() as i64);
        }
    }

    Resp::Integer(0)
}
