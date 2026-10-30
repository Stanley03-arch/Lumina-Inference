use std::sync::atomic::{AtomicU64, Ordering};

/// Lightweight in-process counters (Prometheus export later).
#[derive(Debug, Default)]
pub struct Metrics {
    pub requests_total: AtomicU64,
    pub batches_total: AtomicU64,
    pub batch_items_total: AtomicU64,
    pub errors_total: AtomicU64,
}

impl Metrics {
    pub fn record_request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_batch(&self, size: usize) {
        self.batches_total.fetch_add(1, Ordering::Relaxed);
        self.batch_items_total
            .fetch_add(size as u64, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            batches_total: self.batches_total.load(Ordering::Relaxed),
            batch_items_total: self.batch_items_total.load(Ordering::Relaxed),
            errors_total: self.errors_total.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub batches_total: u64,
    pub batch_items_total: u64,
    pub errors_total: u64,
}
