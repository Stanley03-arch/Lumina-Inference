//! Fixed-window counters for rough QPS / error-rate estimates.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Bucket {
    start: Instant,
    count: u64,
}

/// Counts events in successive fixed windows.
#[derive(Debug, Clone)]
pub struct WindowCounter {
    window: Duration,
    current: Bucket,
    previous: Option<Bucket>,
}

impl WindowCounter {
    pub fn new(window: Duration) -> Self {
        Self {
            window: if window.is_zero() {
                Duration::from_secs(1)
            } else {
                window
            },
            current: Bucket {
                start: Instant::now(),
                count: 0,
            },
            previous: None,
        }
    }

    fn roll(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.current.start) >= self.window {
            self.previous = Some(self.current.clone());
            self.current = Bucket {
                start: now,
                count: 0,
            };
        }
    }

    pub fn incr(&mut self) {
        self.roll();
        self.current.count = self.current.count.saturating_add(1);
    }

    pub fn incr_by(&mut self, n: u64) {
        self.roll();
        self.current.count = self.current.count.saturating_add(n);
    }

    /// Count in the current (possibly partial) window.
    pub fn current_count(&mut self) -> u64 {
        self.roll();
        self.current.count
    }

    /// Count from the last completed window, if any.
    pub fn previous_count(&mut self) -> Option<u64> {
        self.roll();
        self.previous.as_ref().map(|b| b.count)
    }

    /// Estimated rate (events/sec) from the previous full window, else current.
    pub fn rate_per_sec(&mut self) -> f64 {
        self.roll();
        let secs = self.window.as_secs_f64().max(1e-9);
        if let Some(p) = &self.previous {
            p.count as f64 / secs
        } else {
            let elapsed = self.current.start.elapsed().as_secs_f64().max(1e-9);
            self.current.count as f64 / elapsed
        }
    }
}

impl Default for WindowCounter {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        let mut c = WindowCounter::new(Duration::from_secs(60));
        c.incr();
        c.incr_by(2);
        assert_eq!(c.current_count(), 3);
    }
}
