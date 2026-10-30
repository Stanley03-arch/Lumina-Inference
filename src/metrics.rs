use std::sync::atomic::{AtomicU64, Ordering};

/// Lightweight in-process counters with Prometheus text export.
#[derive(Debug, Default)]
pub struct Metrics {
    pub requests_total: AtomicU64,
    pub batches_total: AtomicU64,
    pub batch_items_total: AtomicU64,
    pub errors_total: AtomicU64,
    pub inference_latency_ms_sum: AtomicU64,
    pub inference_latency_ms_count: AtomicU64,
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

    /// Record end-to-end batch latency in milliseconds.
    pub fn record_latency_ms(&self, ms: u64) {
        self.inference_latency_ms_sum
            .fetch_add(ms, Ordering::Relaxed);
        self.inference_latency_ms_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            batches_total: self.batches_total.load(Ordering::Relaxed),
            batch_items_total: self.batch_items_total.load(Ordering::Relaxed),
            errors_total: self.errors_total.load(Ordering::Relaxed),
            inference_latency_ms_sum: self.inference_latency_ms_sum.load(Ordering::Relaxed),
            inference_latency_ms_count: self.inference_latency_ms_count.load(Ordering::Relaxed),
        }
    }

    /// Prometheus text exposition format (0.0.4).
    pub fn encode_prometheus(&self) -> String {
        self.snapshot().encode_prometheus()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub batches_total: u64,
    pub batch_items_total: u64,
    pub errors_total: u64,
    pub inference_latency_ms_sum: u64,
    pub inference_latency_ms_count: u64,
}

impl MetricsSnapshot {
    pub fn encode_prometheus(&self) -> String {
        let mut s = String::with_capacity(512);
        s.push_str("# HELP lumina_requests_total Total inference requests submitted.\n");
        s.push_str("# TYPE lumina_requests_total counter\n");
        s.push_str(&format!("lumina_requests_total {}\n", self.requests_total));

        s.push_str("# HELP lumina_batches_total Total batches executed.\n");
        s.push_str("# TYPE lumina_batches_total counter\n");
        s.push_str(&format!("lumina_batches_total {}\n", self.batches_total));

        s.push_str("# HELP lumina_batch_items_total Total items processed across batches.\n");
        s.push_str("# TYPE lumina_batch_items_total counter\n");
        s.push_str(&format!("lumina_batch_items_total {}\n", self.batch_items_total));

        s.push_str("# HELP lumina_errors_total Total errors during submit/flush.\n");
        s.push_str("# TYPE lumina_errors_total counter\n");
        s.push_str(&format!("lumina_errors_total {}\n", self.errors_total));

        s.push_str("# HELP lumina_inference_latency_ms Inference batch latency in milliseconds.\n");
        s.push_str("# TYPE lumina_inference_latency_ms summary\n");
        s.push_str(&format!(
            "lumina_inference_latency_ms_sum {}\n",
            self.inference_latency_ms_sum
        ));
        s.push_str(&format!(
            "lumina_inference_latency_ms_count {}\n",
            self.inference_latency_ms_count
        ));

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_contains_counters() {
        let m = Metrics::default();
        m.record_request();
        m.record_batch(3);
        m.record_latency_ms(12);
        let text = m.encode_prometheus();
        assert!(text.contains("lumina_requests_total 1"));
        assert!(text.contains("lumina_batches_total 1"));
        assert!(text.contains("lumina_batch_items_total 3"));
        assert!(text.contains("lumina_inference_latency_ms_sum 12"));
        assert!(text.contains("# TYPE lumina_requests_total counter"));
    }
}
