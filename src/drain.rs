//! Graceful drain: stop accepting new work while in-flight completes.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Shared drain / readiness gate.
#[derive(Debug, Clone)]
pub struct DrainGate {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    draining: AtomicBool,
    in_flight: AtomicU64,
}

impl DrainGate {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                draining: AtomicBool::new(false),
                in_flight: AtomicU64::new(0),
            }),
        }
    }

    /// Begin drain — new admits fail with draining error.
    pub fn start_drain(&self) {
        self.inner.draining.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.inner.draining.store(false, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.inner.draining.load(Ordering::SeqCst)
    }

    pub fn in_flight(&self) -> u64 {
        self.inner.in_flight.load(Ordering::SeqCst)
    }

    /// Admit a new request. Returns false when draining.
    pub fn try_enter(&self) -> bool {
        if self.is_draining() {
            return false;
        }
        self.inner.in_flight.fetch_add(1, Ordering::SeqCst);
        // Re-check after increment to reduce race with start_drain.
        if self.is_draining() {
            self.inner.in_flight.fetch_sub(1, Ordering::SeqCst);
            return false;
        }
        true
    }

    pub fn leave(&self) {
        self.inner.in_flight.fetch_sub(1, Ordering::SeqCst);
    }

    /// Block until in-flight hits zero or timeout.
    pub fn wait_idle(&self, timeout: Duration) -> bool {
        let start = Instant::now();
        while self.in_flight() > 0 {
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        true
    }

    pub async fn wait_idle_async(&self, timeout: Duration) -> bool {
        let start = Instant::now();
        while self.in_flight() > 0 {
            if start.elapsed() >= timeout {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        true
    }
}

impl Default for DrainGate {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII token from a successful `try_enter`.
pub struct DrainPermit {
    gate: DrainGate,
}

impl DrainGate {
    pub fn enter(&self) -> Option<DrainPermit> {
        if self.try_enter() {
            Some(DrainPermit {
                gate: self.clone(),
            })
        } else {
            None
        }
    }
}

impl Drop for DrainPermit {
    fn drop(&mut self) {
        self.gate.leave();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_rejects() {
        let g = DrainGate::new();
        let p = g.enter().unwrap();
        g.start_drain();
        assert!(g.enter().is_none());
        drop(p);
        assert_eq!(g.in_flight(), 0);
        assert!(g.wait_idle(Duration::from_millis(50)));
    }
}
