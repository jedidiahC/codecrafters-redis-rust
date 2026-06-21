use crate::{
    resp::Resp,
    store::{RedisStore, StoreElement},
};

use std::{
    cmp::{max, min},
    collections::HashMap,
    time::{Duration, Instant},
};

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

    return Resp::Array(Vec::new());
}
