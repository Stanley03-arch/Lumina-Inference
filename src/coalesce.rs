//! Coalesce duplicate in-flight requests by key (single-flight style).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Tracks keys that already have an in-flight computation.
#[derive(Debug, Default, Clone)]
pub struct Coalescer {
    inflight: Arc<Mutex<HashMap<String, ()>>>,
}

impl Coalescer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Try to claim `key`. Returns `true` if this caller should run the work.
    pub fn try_begin(&self, key: &str) -> bool {
        let mut m = self.inflight.lock().unwrap();
        if m.contains_key(key) {
            return false;
        }
        m.insert(key.to_string(), ());
        true
    }

    pub fn end(&self, key: &str) {
        self.inflight.lock().unwrap().remove(key);
    }

    pub fn is_inflight(&self, key: &str) -> bool {
        self.inflight.lock().unwrap().contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.inflight.lock().unwrap().len()
    }
}

/// RAII guard that ends coalescing on drop.
pub struct CoalesceGuard {
    c: Coalescer,
    key: String,
}

impl Coalescer {
    pub fn begin(&self, key: impl Into<String>) -> Option<CoalesceGuard> {
        let key = key.into();
        if self.try_begin(&key) {
            Some(CoalesceGuard {
                c: self.clone(),
                key,
            })
        } else {
            None
        }
    }
}

impl Drop for CoalesceGuard {
    fn drop(&mut self) {
        self.c.end(&self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_flight() {
        let c = Coalescer::new();
        let g = c.begin("model-a").unwrap();
        assert!(c.begin("model-a").is_none());
        drop(g);
        assert!(c.begin("model-a").is_some());
    }
}
