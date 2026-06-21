use crate::{
    resp::Resp,
    store::{RedisStore, StoreElement},
};

use std::{
    cmp::min,
    collections::HashMap,
    time::{Duration, Instant},
};

pub fn lrange(key: &String, start: usize, end: usize, store: &RedisStore) -> Resp {
    let store = store.lock().unwrap();
    let element = store.get(key);

    if let Some(element) = element {
        if let StoreElement::List { list } = element
            && list.len() > 0
            && start < list.len()
        {
            let end = min(list.len() - 1, end); // If end > length of list, treat end as last element.
            let slice = &list[start..=end];

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
