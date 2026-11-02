//! Exponential backoff calculator for retries.

use crate::jitter::equal_jitter;
use std::time::Duration;

/// `delay = min(cap, base * factor^attempt)` with optional equal jitter.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    pub base: Duration,
    pub factor: f64,
    pub cap: Duration,
    pub jitter: bool,
    attempt: u32,
}

impl ExponentialBackoff {
    pub fn new(base: Duration, factor: f64, cap: Duration) -> Self {
        Self {
            base,
            factor: factor.max(1.0),
            cap,
            jitter: true,
            attempt: 0,
        }
    }

    pub fn with_jitter(mut self, on: bool) -> Self {
        self.jitter = on;
        self
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Advance attempt counter and return the next delay.
    pub fn next_delay(&mut self) -> Duration {
        let exp = self.factor.powi(self.attempt as i32);
        let raw = Duration::from_secs_f64(
            (self.base.as_secs_f64() * exp).min(self.cap.as_secs_f64()),
        );
        self.attempt = self.attempt.saturating_add(1);
        if self.jitter {
            equal_jitter(raw, self.attempt as u64)
        } else {
            raw
        }
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(50),
            2.0,
            Duration::from_secs(30),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows() {
        let mut b = ExponentialBackoff::new(Duration::from_millis(10), 2.0, Duration::from_secs(1))
            .with_jitter(false);
        let d0 = b.next_delay();
        let d1 = b.next_delay();
        assert!(d1 > d0);
    }
}
