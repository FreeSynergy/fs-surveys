//! Shared helpers of the integration tests: a real core with real in-memory stores.

#![allow(dead_code)] // each test file uses a different subset

use std::sync::Arc;

use fs_admin_frontend::{Backend, Record, Values};
use fs_surveys_app::Ctx;

pub fn v(pairs: &[(&str, &[&str])]) -> Values {
    pairs
        .iter()
        .map(|(k, l)| ((*k).to_owned(), l.iter().map(|s| (*s).to_owned()).collect()))
        .collect()
}

pub struct World {
    pub ctx: Arc<Ctx>,
}

impl World {
    pub fn new() -> Self {
        Self {
            ctx: Ctx::in_memory("https://example.org/users/owner", "example.org").unwrap(),
        }
    }
}

pub fn first(b: &dyn Backend, query: &Values) -> Record {
    b.list(query)
        .unwrap()
        .into_iter()
        .next()
        .expect("one record")
}
