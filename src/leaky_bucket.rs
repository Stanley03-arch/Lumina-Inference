//! Leaky-bucket rate limiter (smooth rate, no burst above capacity).

use crate::error::{LuminaError, Result};
use std::time::Instant;

/// Classic leaky bucket: water drains at `rate_per_sec`; admits when level allows.
#[derive(Debug, Clone)]
pub struct LeakyBucket {
    capacity: f64,
    rate_per_sec: f64,
    level: f64,
    last: Instant,
}

impl LeakyBucket {
    pub fn new(rate_per_sec: f64, capacity: f64) -> Self {
        Self {
            capacity: capacity.max(1.0),
            rate_per_sec: rate_per_sec.max(0.001),
            level: 0.0,
            last: Instant::now(),
        }
    }

    fn leak(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f64();
        self.level = (self.level - dt * self.rate_per_sec).max(0.0);
        self.last = now;
    }

    pub fn try_acquire(&mut self, cost: f64) -> Result<()> {
        self.leak();
        let cost = cost.max(0.0);
        if self.level + cost <= self.capacity {
            self.level += cost;
            Ok(())
        } else {
            Err(LuminaError::RateLimited)
        }
    }

    pub fn level(&mut self) -> f64 {
        self.leak();
        self.level
    }
}

impl Default for LeakyBucket {
    fn default() -> Self {
        Self::new(100.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits() {
        let mut b = LeakyBucket::new(1.0, 2.0);
        assert!(b.try_acquire(1.0).is_ok());
        assert!(b.try_acquire(1.0).is_ok());
        assert!(b.try_acquire(1.0).is_err());
    }
}
