//! Retry with exponential backoff (+ optional jitter).

use crate::error::{LuminaError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

/// Policy for transient failure retries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub multiplier: f64,
    /// Add up to this fraction of backoff as jitter (0.0–1.0).
    pub jitter: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 50,
            max_backoff_ms: 2_000,
            multiplier: 2.0,
            jitter: 0.2,
        }
    }
}

impl RetryPolicy {
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        // attempt is 0-based after first failure
        let base = self.initial_backoff_ms as f64 * self.multiplier.powi(attempt as i32);
        let capped = base.min(self.max_backoff_ms as f64);
        let jitter_span = capped * self.jitter.clamp(0.0, 1.0);
        // Deterministic-ish jitter from attempt (no RNG dependency).
        let j = ((attempt.wrapping_mul(2654435761)) % 1000) as f64 / 1000.0 * jitter_span;
        Duration::from_millis((capped + j) as u64)
    }
}

/// Run a fallible closure with retries. Sleeps between attempts on error.
pub fn retry<T, F>(policy: &RetryPolicy, mut f: F) -> Result<T>
where
    F: FnMut(u32) -> Result<T>,
{
    let attempts = policy.max_attempts.max(1);
    let mut last = LuminaError::Internal("retry: no attempts".into());
    for attempt in 0..attempts {
        match f(attempt) {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = e;
                if attempt + 1 >= attempts {
                    break;
                }
                let wait = policy.backoff_for_attempt(attempt);
                debug!(attempt, wait_ms = wait.as_millis(), "retry backoff");
                std::thread::sleep(wait);
            }
        }
    }
    Err(last)
}

/// Async variant using `tokio::time::sleep`.
pub async fn retry_async<T, F, Fut>(policy: &RetryPolicy, mut f: F) -> Result<T>
where
    F: FnMut(u32) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let attempts = policy.max_attempts.max(1);
    let mut last = LuminaError::Internal("retry: no attempts".into());
    for attempt in 0..attempts {
        match f(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = e;
                if attempt + 1 >= attempts {
                    break;
                }
                let wait = policy.backoff_for_attempt(attempt);
                debug!(attempt, wait_ms = wait.as_millis(), "async retry backoff");
                tokio::time::sleep(wait).await;
            }
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn succeeds_after_failures() {
        let n = AtomicU32::new(0);
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_backoff_ms: 1,
            max_backoff_ms: 5,
            multiplier: 2.0,
            jitter: 0.0,
        };
        let out = retry(&policy, |_| {
            let i = n.fetch_add(1, Ordering::SeqCst);
            if i < 2 {
                Err(LuminaError::Internal("fail".into()))
            } else {
                Ok(42)
            }
        })
        .unwrap();
        assert_eq!(out, 42);
        assert_eq!(n.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn backoff_grows() {
        let p = RetryPolicy::default();
        assert!(p.backoff_for_attempt(1) >= p.backoff_for_attempt(0));
    }
}
