//! Per-worker circuit breaker.
//!
//! After consecutive failures a peer is opened (skipped) until a cool-down
//! elapses, then half-open for a trial request.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Circuit state for one worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
struct Entry {
    failures: u32,
    successes: u32,
    opened_at: Option<Instant>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            failures: 0,
            successes: 0,
            opened_at: None,
        }
    }
}

/// Tunables for the breaker.
#[derive(Debug, Clone)]
pub struct CircuitConfig {
    /// Failures before opening.
    pub failure_threshold: u32,
    /// Successes in half-open before closing.
    pub success_threshold: u32,
    /// Time to stay open before a trial.
    pub open_timeout: Duration,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 1,
            open_timeout: Duration::from_secs(30),
        }
    }
}

/// Tracks circuit state keyed by worker id.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    cfg: CircuitConfig,
    entries: HashMap<String, Entry>,
}

impl CircuitBreaker {
    pub fn new(cfg: CircuitConfig) -> Self {
        Self {
            cfg,
            entries: HashMap::new(),
        }
    }

    pub fn config(&self) -> &CircuitConfig {
        &self.cfg
    }

    pub fn state(&self, id: &str) -> CircuitState {
        let Some(e) = self.entries.get(id) else {
            return CircuitState::Closed;
        };
        match e.opened_at {
            None => CircuitState::Closed,
            Some(t) if t.elapsed() >= self.cfg.open_timeout => CircuitState::HalfOpen,
            Some(_) => CircuitState::Open,
        }
    }

    /// Whether the scheduler may dispatch to this worker.
    pub fn allow(&self, id: &str) -> bool {
        !matches!(self.state(id), CircuitState::Open)
    }

    pub fn record_success(&mut self, id: &str) {
        let e = self.entries.entry(id.to_string()).or_default();
        match self.state_of(e) {
            CircuitState::HalfOpen => {
                e.successes += 1;
                if e.successes >= self.cfg.success_threshold {
                    e.failures = 0;
                    e.successes = 0;
                    e.opened_at = None;
                }
            }
            CircuitState::Closed => {
                e.failures = 0;
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&mut self, id: &str) {
        let threshold = self.cfg.failure_threshold;
        let e = self.entries.entry(id.to_string()).or_default();
        e.failures = e.failures.saturating_add(1);
        e.successes = 0;
        if e.failures >= threshold {
            e.opened_at = Some(Instant::now());
        }
    }

    pub fn reset(&mut self, id: &str) {
        self.entries.remove(id);
    }

    fn state_of(&self, e: &Entry) -> CircuitState {
        match e.opened_at {
            None => CircuitState::Closed,
            Some(t) if t.elapsed() >= self.cfg.open_timeout => CircuitState::HalfOpen,
            Some(_) => CircuitState::Open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_threshold() {
        let mut cb = CircuitBreaker::new(CircuitConfig {
            failure_threshold: 2,
            success_threshold: 1,
            open_timeout: Duration::from_secs(60),
        });
        assert!(cb.allow("w"));
        cb.record_failure("w");
        assert!(cb.allow("w"));
        cb.record_failure("w");
        assert!(!cb.allow("w"));
        assert_eq!(cb.state("w"), CircuitState::Open);
    }

    #[test]
    fn half_open_then_close() {
        let mut cb = CircuitBreaker::new(CircuitConfig {
            failure_threshold: 1,
            success_threshold: 1,
            open_timeout: Duration::from_millis(1),
        });
        cb.record_failure("w");
        assert_eq!(cb.state("w"), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(cb.state("w"), CircuitState::HalfOpen);
        assert!(cb.allow("w"));
        cb.record_success("w");
        assert_eq!(cb.state("w"), CircuitState::Closed);
    }
}
