//! Sampling gates for tracing / expensive diagnostics.

use std::sync::atomic::{AtomicU64, Ordering};

/// Pass every N-th call (deterministic).
#[derive(Debug)]
pub struct EveryN {
    n: u64,
    counter: AtomicU64,
}

impl EveryN {
    pub fn new(n: u64) -> Self {
        Self {
            n: n.max(1),
            counter: AtomicU64::new(0),
        }
    }

    pub fn hit(&self) -> bool {
        let c = self.counter.fetch_add(1, Ordering::Relaxed);
        c % self.n == 0
    }
}

/// Approximate percentage sampler using a counter (not cryptographic RNG).
#[derive(Debug)]
pub struct PercentSampler {
    percent: u8,
    counter: AtomicU64,
}

impl PercentSampler {
    pub fn new(percent: u8) -> Self {
        Self {
            percent: percent.min(100),
            counter: AtomicU64::new(0),
        }
    }

    pub fn hit(&self) -> bool {
        if self.percent >= 100 {
            return true;
        }
        if self.percent == 0 {
            return false;
        }
        let c = self.counter.fetch_add(1, Ordering::Relaxed);
        (c % 100) < self.percent as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_n() {
        let e = EveryN::new(3);
        assert!(e.hit());
        assert!(!e.hit());
        assert!(!e.hit());
        assert!(e.hit());
    }

    #[test]
    fn percent_100() {
        let p = PercentSampler::new(100);
        assert!(p.hit());
    }
}
