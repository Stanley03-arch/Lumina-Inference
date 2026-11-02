//! Throttle: enforce a minimum interval between successful admits.

use std::time::{Duration, Instant};

/// Allows at most one success per `min_interval`.
#[derive(Debug, Clone)]
pub struct Throttle {
    min_interval: Duration,
    last: Option<Instant>,
}

impl Throttle {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            last: None,
        }
    }

    /// Returns `true` if enough time has elapsed since the last success.
    pub fn try_admit(&mut self) -> bool {
        let now = Instant::now();
        if let Some(t) = self.last {
            if now.duration_since(t) < self.min_interval {
                return false;
            }
        }
        self.last = Some(now);
        true
    }

    pub fn time_until_ready(&self) -> Duration {
        match self.last {
            None => Duration::ZERO,
            Some(t) => self
                .min_interval
                .saturating_sub(t.elapsed()),
        }
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

impl Default for Throttle {
    fn default() -> Self {
        Self::new(Duration::from_millis(10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_second() {
        let mut t = Throttle::new(Duration::from_secs(60));
        assert!(t.try_admit());
        assert!(!t.try_admit());
    }
}
