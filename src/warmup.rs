//! Warm-up helpers: send dummy batches so models load kernels / caches.

use crate::batch::InferenceRequest;
use crate::error::Result;
use crate::worker::InferenceWorker;
use std::time::Instant;
use tracing::info;

/// Warm-up options.
#[derive(Debug, Clone)]
pub struct WarmupConfig {
    pub model: String,
    /// Bytes for the dummy primary input.
    pub input_bytes: Vec<u8>,
    /// Number of warm-up iterations.
    pub iterations: usize,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            model: "default".into(),
            input_bytes: vec![0u8; 16],
            iterations: 3,
        }
    }
}

/// Run warm-up batches against a worker; returns total wall time ms.
pub fn warmup_worker(worker: &dyn InferenceWorker, cfg: &WarmupConfig) -> Result<u64> {
    let start = Instant::now();
    for i in 0..cfg.iterations.max(1) {
        let req = InferenceRequest::new(format!("warmup-{i}"), cfg.model.clone(), cfg.input_bytes.clone());
        let _ = worker.run_batch(&[req])?;
    }
    let ms = start.elapsed().as_millis() as u64;
    info!(
        worker = worker.name(),
        iterations = cfg.iterations,
        ms,
        "warmup complete"
    );
    Ok(ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker::LocalWorker;

    #[test]
    fn warms_local() {
        let w = LocalWorker::new("local");
        let ms = warmup_worker(&w, &WarmupConfig::default()).unwrap();
        assert!(ms < 60_000);
    }
}
