//! Queue depth gauge with high / low watermarks.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Tracks depth and whether high watermark was crossed.
#[derive(Debug)]
pub struct DepthGauge {
    depth: AtomicUsize,
    high: usize,
    low: usize,
    overloaded: AtomicUsize, // 0/1 as bool
}

impl DepthGauge {
    pub fn new(high: usize, low: usize) -> Self {
        let high = high.max(1);
        let low = low.min(high);
        Self {
            depth: AtomicUsize::new(0),
            high,
            low,
            overloaded: AtomicUsize::new(0),
        }
    }

    pub fn set(&self, depth: usize) {
        self.depth.store(depth, Ordering::Relaxed);
        if depth >= self.high {
            self.overloaded.store(1, Ordering::Relaxed);
        } else if depth <= self.low {
            self.overloaded.store(0, Ordering::Relaxed);
        }
    }

    pub fn incr(&self) -> usize {
        let d = self.depth.fetch_add(1, Ordering::Relaxed) + 1;
        self.set(d);
        d
    }

    pub fn decr(&self) -> usize {
        let d = self
            .depth
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(v.saturating_sub(1))
            })
            .unwrap_or(0);
        let d = d.saturating_sub(1);
        self.set(d);
        d
    }

    pub fn depth(&self) -> usize {
        self.depth.load(Ordering::Relaxed)
    }

    pub fn is_overloaded(&self) -> bool {
        self.overloaded.load(Ordering::Relaxed) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watermarks() {
        let g = DepthGauge::new(5, 2);
        for _ in 0..5 {
            g.incr();
        }
        assert!(g.is_overloaded());
        for _ in 0..4 {
            g.decr();
        }
        assert!(!g.is_overloaded());
    }
}
