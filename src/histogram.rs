//! Fixed-bucket latency histogram (milliseconds).

use std::sync::atomic::{AtomicU64, Ordering};

/// Default upper bounds in ms for latency buckets.
pub const DEFAULT_BOUNDS_MS: &[u64] = &[1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000];

/// Thread-safe histogram with fixed upper bounds (+Inf).
#[derive(Debug)]
pub struct Histogram {
    bounds: Vec<u64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
}

impl Histogram {
    pub fn new(bounds_ms: &[u64]) -> Self {
        let mut bounds = bounds_ms.to_vec();
        bounds.sort_unstable();
        bounds.dedup();
        let n = bounds.len() + 1; // +Inf
        let mut counts = Vec::with_capacity(n);
        for _ in 0..n {
            counts.push(AtomicU64::new(0));
        }
        Self {
            bounds,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    pub fn default_latency() -> Self {
        Self::new(DEFAULT_BOUNDS_MS)
    }

    pub fn observe(&self, value_ms: u64) {
        let mut idx = self.bounds.len(); // +Inf
        for (i, &b) in self.bounds.iter().enumerate() {
            if value_ms <= b {
                idx = i;
                break;
            }
        }
        self.counts[idx].fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value_ms, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }

    /// Approximate percentile (0.0–1.0) from bucket cumulative counts.
    pub fn percentile(&self, p: f64) -> Option<u64> {
        let total = self.count() as f64;
        if total == 0.0 {
            return None;
        }
        let target = (p.clamp(0.0, 1.0) * total).ceil().max(1.0);
        let mut cum = 0u64;
        for (i, c) in self.counts.iter().enumerate() {
            cum += c.load(Ordering::Relaxed);
            if cum as f64 >= target {
                return Some(if i < self.bounds.len() {
                    self.bounds[i]
                } else {
                    self.bounds.last().copied().unwrap_or(0).saturating_mul(2)
                });
            }
        }
        self.bounds.last().copied()
    }

    pub fn p50(&self) -> Option<u64> {
        self.percentile(0.50)
    }
    pub fn p95(&self) -> Option<u64> {
        self.percentile(0.95)
    }
    pub fn p99(&self) -> Option<u64> {
        self.percentile(0.99)
    }

    /// Prometheus histogram exposition (`le` buckets).
    pub fn encode_prometheus(&self, name: &str, help: &str) -> String {
        let mut s = String::new();
        s.push_str(&format!("# HELP {name} {help}\n"));
        s.push_str(&format!("# TYPE {name} histogram\n"));
        let mut cum = 0u64;
        for (i, &b) in self.bounds.iter().enumerate() {
            cum += self.counts[i].load(Ordering::Relaxed);
            s.push_str(&format!("{name}_bucket{{le=\"{b}\"}} {cum}\n"));
        }
        cum += self.counts[self.bounds.len()].load(Ordering::Relaxed);
        s.push_str(&format!("{name}_bucket{{le=\"+Inf\"}} {cum}\n"));
        s.push_str(&format!("{name}_sum {}\n", self.sum()));
        s.push_str(&format!("{name}_count {}\n", self.count()));
        s
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::default_latency()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles() {
        let h = Histogram::new(&[10, 50, 100]);
        for _ in 0..90 {
            h.observe(5);
        }
        for _ in 0..9 {
            h.observe(40);
        }
        h.observe(80);
        assert_eq!(h.p50(), Some(10));
        assert!(h.p95().unwrap() >= 50);
    }
}
