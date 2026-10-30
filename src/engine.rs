use crate::batch::{BatchAccumulator, InferenceRequest, InferenceResponse};
use crate::config::EngineConfig;
use crate::error::Result;
use crate::metrics::{Metrics, MetricsSnapshot};
use crate::worker::{InferenceWorker, LocalWorker};
use std::sync::Arc;
use tracing::{info, instrument};

/// Central coordinator: accumulate → flush → worker.
pub struct InferenceEngine {
    cfg: EngineConfig,
    batch: BatchAccumulator,
    workers: Vec<Box<dyn InferenceWorker>>,
    next_worker: usize,
    metrics: Arc<Metrics>,
}

impl InferenceEngine {
    pub fn new(cfg: EngineConfig) -> Result<Self> {
        Self::with_metrics(cfg, Arc::new(Metrics::default()))
    }

    pub fn with_metrics(cfg: EngineConfig, metrics: Arc<Metrics>) -> Result<Self> {
        cfg.validate()?;
        let batch = BatchAccumulator::new(&cfg);
        let workers: Vec<Box<dyn InferenceWorker>> = (0..cfg.worker_slots)
            .map(|i| Box::new(LocalWorker::new(format!("worker-{i}"))) as Box<dyn InferenceWorker>)
            .collect();
        Ok(Self {
            cfg,
            batch,
            workers,
            next_worker: 0,
            metrics,
        })
    }

    /// Build an engine with an explicit worker pool (e.g. ONNX workers).
    pub fn with_workers(
        cfg: EngineConfig,
        workers: Vec<Box<dyn InferenceWorker>>,
        metrics: Arc<Metrics>,
    ) -> Result<Self> {
        cfg.validate()?;
        if workers.is_empty() {
            return Err(crate::LuminaError::Config(
                "at least one worker is required".into(),
            ));
        }
        let batch = BatchAccumulator::new(&cfg);
        Ok(Self {
            cfg,
            batch,
            workers,
            next_worker: 0,
            metrics,
        })
    }

    pub fn config(&self) -> &EngineConfig {
        &self.cfg
    }

    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    pub fn metrics_handle(&self) -> Arc<Metrics> {
        Arc::clone(&self.metrics)
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// Enqueue a request; returns completed batch results when full.
    #[instrument(skip(self, req), fields(id = %req.id, model = %req.model))]
    pub fn submit(&mut self, req: InferenceRequest) -> Result<Option<Vec<InferenceResponse>>> {
        self.metrics.record_request();
        if let Err(e) = self.batch.push(req) {
            self.metrics.record_error();
            return Err(e);
        }
        if self.batch.is_full() {
            return Ok(Some(self.flush()?));
        }
        Ok(None)
    }

    /// Force-run whatever is currently buffered.
    pub fn flush(&mut self) -> Result<Vec<InferenceResponse>> {
        if self.batch.is_empty() {
            return Ok(Vec::new());
        }
        let items = self.batch.take();
        let n = items.len();
        let idx = self.next_worker % self.workers.len();
        self.next_worker = self.next_worker.wrapping_add(1);
        let worker = &self.workers[idx];
        info!(worker = %worker.name(), n, "running batch");
        match worker.run_batch(&items) {
            Ok(out) => {
                self.metrics.record_batch(n);
                Ok(out)
            }
            Err(e) => {
                self.metrics.record_error();
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flushes_when_full() {
        let cfg = EngineConfig {
            max_batch_size: 2,
            worker_slots: 1,
            ..Default::default()
        };
        let mut eng = InferenceEngine::new(cfg).unwrap();
        let r1 = eng
            .submit(InferenceRequest {
                id: "a".into(),
                model: "m".into(),
                input: b"hi".to_vec(),
            })
            .unwrap();
        assert!(r1.is_none());
        let r2 = eng
            .submit(InferenceRequest {
                id: "b".into(),
                model: "m".into(),
                input: b"yo".to_vec(),
            })
            .unwrap();
        let out = r2.expect("batch should flush");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].output, b"hi");
        let snap = eng.metrics();
        assert_eq!(snap.requests_total, 2);
        assert_eq!(snap.batches_total, 1);
        assert_eq!(snap.batch_items_total, 2);
    }
}
