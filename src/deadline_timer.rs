//! Absolute deadline helper for request budgets.

use crate::error::{LuminaError, Result};
use std::time::{Duration, Instant};

/// Tracks an absolute deadline from a start instant.
#[derive(Debug, Clone, Copy)]
pub struct DeadlineTimer {
    start: Instant,
    deadline: Instant,
}

impl DeadlineTimer {
    pub fn from_now(budget: Duration) -> Self {
        let start = Instant::now();
        Self {
            start,
            deadline: start + budget,
        }
    }

    pub fn from_start(start: Instant, budget: Duration) -> Self {
        Self {
            start,
            deadline: start + budget,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn check(&self) -> Result<()> {
        if self.is_expired() {
            Err(LuminaError::DeadlineExceeded)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_expired_immediately() {
        let d = DeadlineTimer::from_now(Duration::from_secs(5));
        assert!(!d.is_expired());
        assert!(d.check().is_ok());
    }

    #[test]
    fn expired_zero() {
        let d = DeadlineTimer::from_now(Duration::ZERO);
        assert!(d.is_expired());
        assert!(matches!(d.check(), Err(LuminaError::DeadlineExceeded)));
    }
}
