use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub type RedisStore = Arc<Mutex<HashMap<String, StoreElement>>>;

pub enum StoreElement {
    String {
        value: String,
        expiration: Option<Instant>,
    },
    List {
        value: Vec<String>,
    },
}
