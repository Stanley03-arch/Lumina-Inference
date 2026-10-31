//! Token-bucket rate limiter for request admission control.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Configuration for a token bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Sustained rate (tokens per second).
    pub rate_per_sec: f64,
    /// Maximum burst size.
    pub burst: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            rate_per_sec: 100.0,
            burst: 200.0,
        }
    }
}

/// Simple token-bucket limiter (single-threaded / external mutex).
#[derive(Debug, Clone)]
pub struct RateLimiter {
    cfg: RateLimitConfig,
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    pub fn new(cfg: RateLimitConfig) -> Self {
        Self {
            tokens: cfg.burst,
            cfg,
            last: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last.elapsed().as_secs_f64();
        self.last = Instant::now();
        self.tokens = (self.tokens + elapsed * self.cfg.rate_per_sec).min(self.cfg.burst);
    }

    /// Try to consume `n` tokens. Returns `true` if allowed.
    pub fn try_acquire(&mut self, n: f64) -> bool {
        self.refill();
        if self.tokens >= n {
            self.tokens -= n;
            true
        } else {
            false
        }
    }

    /// Consume one token or return false.
    pub fn allow(&mut self) -> bool {
        self.try_acquire(1.0)
    }

    pub fn available(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    pub fn config(&self) -> &RateLimitConfig {
        &self.cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_then_block() {
        let mut lim = RateLimiter::new(RateLimitConfig {
            rate_per_sec: 10.0,
            burst: 3.0,
        });
        assert!(lim.allow());
        assert!(lim.allow());
        assert!(lim.allow());
        assert!(!lim.allow());
    }
}
