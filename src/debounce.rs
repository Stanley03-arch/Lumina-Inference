//! Debounce: only allow an action once per quiet period.

use std::time::{Duration, Instant};

/// Returns true at most once per `interval`.
#[derive(Debug, Clone)]
pub struct Debounce {
    interval: Duration,
    last: Option<Instant>,
}

impl Debounce {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: None,
        }
    }

    /// `true` if enough time has passed since the last success.
    pub fn allow(&mut self) -> bool {
        let now = Instant::now();
        if let Some(t) = self.last {
            if now.duration_since(t) < self.interval {
                return false;
            }
        }
        self.last = Some(now);
        true
    }

    pub fn reset(&mut self) {
        self.last = None;
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Default for Debounce {
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_ok_second_blocked() {
        let mut d = Debounce::new(Duration::from_secs(60));
        assert!(d.allow());
        assert!(!d.allow());
    }
}
