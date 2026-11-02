//! Per-key cooldown after failures (e.g. bad peer).

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Marks keys as cooling down for a fixed duration.
#[derive(Debug, Clone, Default)]
pub struct Cooldown {
    until: HashMap<String, Instant>,
    duration: Duration,
}

impl Cooldown {
    pub fn new(duration: Duration) -> Self {
        Self {
            until: HashMap::new(),
            duration,
        }
    }

    pub fn trip(&mut self, key: impl Into<String>) {
        self.until
            .insert(key.into(), Instant::now() + self.duration);
    }

    pub fn is_cooling(&self, key: &str) -> bool {
        self.until
            .get(key)
            .map(|t| Instant::now() < *t)
            .unwrap_or(false)
    }

    pub fn clear(&mut self, key: &str) {
        self.until.remove(key);
    }

    pub fn purge_expired(&mut self) {
        let now = Instant::now();
        self.until.retain(|_, t| *t > now);
    }

    pub fn len(&self) -> usize {
        self.until.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trips() {
        let mut c = Cooldown::new(Duration::from_secs(60));
        assert!(!c.is_cooling("gpu-a"));
        c.trip("gpu-a");
        assert!(c.is_cooling("gpu-a"));
    }
}
