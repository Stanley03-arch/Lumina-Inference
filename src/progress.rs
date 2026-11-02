//! Progress tracking for long-running loads / warm-ups.

use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic completed / total progress.
#[derive(Debug, Default)]
pub struct Progress {
    done: AtomicU64,
    total: AtomicU64,
}

impl Progress {
    pub fn new(total: u64) -> Self {
        Self {
            done: AtomicU64::new(0),
            total: AtomicU64::new(total),
        }
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn incr(&self) {
        self.done.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_by(&self, n: u64) {
        self.done.fetch_add(n, Ordering::Relaxed);
    }

    pub fn set_done(&self, n: u64) {
        self.done.store(n, Ordering::Relaxed);
    }

    pub fn done(&self) -> u64 {
        self.done.load(Ordering::Relaxed)
    }

    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    /// 0.0–100.0; 100 if total is 0.
    pub fn percent(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            return 100.0;
        }
        (self.done() as f64 / t as f64 * 100.0).min(100.0)
    }

    pub fn is_complete(&self) -> bool {
        let t = self.total();
        t > 0 && self.done() >= t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent() {
        let p = Progress::new(4);
        p.incr();
        p.incr();
        assert!((p.percent() - 50.0).abs() < 1e-9);
    }
}
