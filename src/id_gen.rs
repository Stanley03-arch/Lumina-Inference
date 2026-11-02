//! Monotonic request / batch ID generator.

use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe monotonic counter with optional prefix.
#[derive(Debug)]
pub struct IdGen {
    prefix: String,
    counter: AtomicU64,
}

impl IdGen {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            counter: AtomicU64::new(1),
        }
    }

    pub fn next(&self) -> String {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        if self.prefix.is_empty() {
            n.to_string()
        } else {
            format!("{}-{n}", self.prefix)
        }
    }

    pub fn next_u64(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    pub fn peek(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}

impl Default for IdGen {
    fn default() -> Self {
        Self::new("req")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_ids() {
        let g = IdGen::new("batch");
        let a = g.next();
        let b = g.next();
        assert_ne!(a, b);
        assert!(a.starts_with("batch-"));
    }
}
