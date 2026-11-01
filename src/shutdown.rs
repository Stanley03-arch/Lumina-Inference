//! Cooperative shutdown signal shared across tasks.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Broadcast-style shutdown flag.
#[derive(Debug, Clone)]
pub struct Shutdown {
    flag: Arc<AtomicBool>,
}

impl Shutdown {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Reset for tests / reuse.
    pub fn reset(&self) {
        self.flag.store(false, Ordering::SeqCst);
    }

    /// Poll until triggered or timeout; returns true if triggered.
    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while !self.is_triggered() {
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        true
    }

    pub async fn wait_timeout_async(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while !self.is_triggered() {
            if start.elapsed() >= timeout {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        true
    }

    /// Wait for Ctrl+C then trigger (async).
    pub async fn wait_ctrl_c(self) {
        let _ = tokio::signal::ctrl_c().await;
        self.trigger();
        tracing::info!("shutdown: ctrl-c received");
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_visible() {
        let s = Shutdown::new();
        assert!(!s.is_triggered());
        s.trigger();
        assert!(s.is_triggered());
    }
}
