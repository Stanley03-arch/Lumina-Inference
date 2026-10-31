//! Sliding-window SLO tracker (success rate + latency budget).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct Sample {
    at: Instant,
    ok: bool,
    latency_ms: u64,
}

/// Tracks recent outcomes against an SLO target.
#[derive(Debug, Clone)]
pub struct SloTracker {
    window: Duration,
    samples: VecDeque<Sample>,
    /// Minimum success ratio (e.g. 0.99).
    pub min_success_ratio: f64,
    /// Max allowed latency for "good" samples (ms).
    pub max_latency_ms: u64,
}

impl SloTracker {
    pub fn new(window: Duration, min_success_ratio: f64, max_latency_ms: u64) -> Self {
        Self {
            window,
            samples: VecDeque::new(),
            min_success_ratio,
            max_latency_ms,
        }
    }

    pub fn record(&mut self, ok: bool, latency_ms: u64) {
        self.samples.push_back(Sample {
            at: Instant::now(),
            ok,
            latency_ms,
        });
        self.evict();
    }

    fn evict(&mut self) {
        let cutoff = Instant::now() - self.window;
        while self.samples.front().map(|s| s.at < cutoff).unwrap_or(false) {
            self.samples.pop_front();
        }
    }

    pub fn sample_count(&mut self) -> usize {
        self.evict();
        self.samples.len()
    }

    pub fn success_ratio(&mut self) -> Option<f64> {
        self.evict();
        if self.samples.is_empty() {
            return None;
        }
        let ok = self.samples.iter().filter(|s| s.ok).count() as f64;
        Some(ok / self.samples.len() as f64)
    }

    /// Fraction of samples within latency budget **and** successful.
    pub fn good_ratio(&mut self) -> Option<f64> {
        self.evict();
        if self.samples.is_empty() {
            return None;
        }
        let good = self
            .samples
            .iter()
            .filter(|s| s.ok && s.latency_ms <= self.max_latency_ms)
            .count() as f64;
        Some(good / self.samples.len() as f64)
    }

    /// `true` when enough samples and SLO is met.
    pub fn is_healthy(&mut self, min_samples: usize) -> bool {
        self.evict();
        if self.samples.len() < min_samples {
            return true; // not enough data — optimistically healthy
        }
        self.good_ratio()
            .map(|r| r >= self.min_success_ratio)
            .unwrap_or(true)
    }
}

impl Default for SloTracker {
    fn default() -> Self {
        Self::new(Duration::from_secs(60), 0.99, 500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_breach() {
        let mut t = SloTracker::new(Duration::from_secs(60), 0.9, 100);
        for _ in 0..10 {
            t.record(true, 10);
        }
        assert!(t.is_healthy(5));
        for _ in 0..10 {
            t.record(false, 10);
        }
        assert!(!t.is_healthy(5));
    }
}
