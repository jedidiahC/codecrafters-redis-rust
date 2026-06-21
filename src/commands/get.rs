use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::store::{RedisStore, StoreElement};

use super::Resp;

pub fn get(key: &String, store: &RedisStore) -> Resp {
    let mut store = store.lock().unwrap();

    if let Some(store_element) = store.get(key) {
        if let StoreElement::String { value, expiration } = store_element {
            if let Some(expiration) = expiration {
                // Already expired.
                if &Instant::now() > expiration {
                    store.remove(key);
                    return Resp::Null;
                }
            }

            return Resp::BulkString(value.clone());
        }

        // TODO: Should return error if value is not string.
    }

    Resp::Null
}
