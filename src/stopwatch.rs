//! Stopwatch for measuring elapsed wall time.

use std::time::{Duration, Instant};

/// Simple start/stop timer.
#[derive(Debug, Clone)]
pub struct Stopwatch {
    start: Instant,
    elapsed: Duration,
    running: bool,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            elapsed: Duration::ZERO,
            running: false,
        }
    }

    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            elapsed: Duration::ZERO,
            running: true,
        }
    }

    pub fn begin(&mut self) {
        self.start = Instant::now();
        self.running = true;
    }

    pub fn stop(&mut self) -> Duration {
        if self.running {
            self.elapsed += self.start.elapsed();
            self.running = false;
        }
        self.elapsed
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.running = false;
    }

    pub fn elapsed(&self) -> Duration {
        if self.running {
            self.elapsed + self.start.elapsed()
        } else {
            self.elapsed
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that records duration on drop via a callback-style return.
pub struct ScopeTimer {
    start: Instant,
}

impl ScopeTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures() {
        let mut s = Stopwatch::start();
        std::thread::sleep(Duration::from_millis(5));
        assert!(s.elapsed_ms() >= 1);
        s.stop();
    }
}
