//! Sticky affinity: map a client/session key to a preferred worker id.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Entry {
    worker_id: String,
    last_used: Instant,
}

/// Session → worker affinity with optional TTL eviction.
#[derive(Debug, Clone)]
pub struct AffinityTable {
    map: HashMap<String, Entry>,
    ttl: Duration,
    max_entries: usize,
}

impl AffinityTable {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            map: HashMap::new(),
            ttl,
            max_entries: max_entries.max(1),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        self.evict_expired();
        if let Some(e) = self.map.get_mut(key) {
            e.last_used = Instant::now();
            return Some(e.worker_id.clone());
        }
        None
    }

    pub fn bind(&mut self, key: impl Into<String>, worker_id: impl Into<String>) {
        self.evict_expired();
        if self.map.len() >= self.max_entries && !self.map.contains_key(key.as_ref()) {
            // Drop least-recently used
            if let Some(oldest) = self
                .map
                .iter()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(k, _)| k.clone())
            {
                self.map.remove(&oldest);
            }
        }
        self.map.insert(
            key.into(),
            Entry {
                worker_id: worker_id.into(),
                last_used: Instant::now(),
            },
        );
    }

    pub fn unbind(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn evict_expired(&mut self) {
        let ttl = self.ttl;
        self.map.retain(|_, e| e.last_used.elapsed() < ttl);
    }
}

impl Default for AffinityTable {
    fn default() -> Self {
        Self::new(Duration::from_secs(300), 10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticky() {
        let mut t = AffinityTable::new(Duration::from_secs(60), 10);
        assert!(t.get("c1").is_none());
        t.bind("c1", "gpu-a");
        assert_eq!(t.get("c1").as_deref(), Some("gpu-a"));
    }
}
