//! Per-key token-bucket rate limits (e.g. by client id).

use crate::error::{LuminaError, Result};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last: Instant,
}

/// Independent token buckets keyed by string (client / tenant).
#[derive(Debug, Clone)]
pub struct KeyedLimiter {
    map: HashMap<String, Bucket>,
    rate_per_sec: f64,
    burst: f64,
    max_keys: usize,
}

impl KeyedLimiter {
    pub fn new(rate_per_sec: f64, burst: f64, max_keys: usize) -> Self {
        Self {
            map: HashMap::new(),
            rate_per_sec: rate_per_sec.max(0.001),
            burst: burst.max(1.0),
            max_keys: max_keys.max(1),
        }
    }

    fn refill(b: &mut Bucket, rate: f64, burst: f64) {
        let now = Instant::now();
        let dt = now.duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + dt * rate).min(burst);
        b.last = now;
    }

    pub fn try_acquire(&mut self, key: &str) -> Result<()> {
        if !self.map.contains_key(key) && self.map.len() >= self.max_keys {
            // Evict an arbitrary key (first)
            if let Some(k) = self.map.keys().next().cloned() {
                self.map.remove(&k);
            }
        }
        let rate = self.rate_per_sec;
        let burst = self.burst;
        let b = self.map.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: burst,
            last: Instant::now(),
        });
        Self::refill(b, rate, burst);
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            Ok(())
        } else {
            Err(LuminaError::RateLimited)
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl Default for KeyedLimiter {
    fn default() -> Self {
        Self::new(10.0, 20.0, 10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_burst() {
        let mut k = KeyedLimiter::new(1.0, 2.0, 10);
        assert!(k.try_acquire("a").is_ok());
        assert!(k.try_acquire("a").is_ok());
        assert!(k.try_acquire("a").is_err());
    }
}
