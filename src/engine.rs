use crate::batch::{BatchAccumulator, InferenceRequest, InferenceResponse};
use crate::config::EngineConfig;
use crate::error::Result;
use crate::worker::LocalWorker;
use tracing::{info, instrument};

/// Central coordinator: accumulate → flush → worker.
pub struct InferenceEngine {
    cfg: EngineConfig,
    batch: BatchAccumulator,
    workers: Vec<LocalWorker>,
    next_worker: usize,
}

impl InferenceEngine {
    pub fn new(cfg: EngineConfig) -> Result<Self> {
        cfg.validate()?;
        let batch = BatchAccumulator::new(&cfg);
        let workers = (0..cfg.worker_slots)
            .map(|i| LocalWorker::new(format!("worker-{i}")))
            .collect();
        Ok(Self {
            cfg,
            batch,
            workers,
            next_worker: 0,
        })
    }

    pub fn config(&self) -> &EngineConfig {
        &self.cfg
    }

    /// Enqueue a request; returns completed batch results when full.
    #[instrument(skip(self, req), fields(id = %req.id, model = %req.model))]
    pub fn submit(&mut self, req: InferenceRequest) -> Result<Option<Vec<InferenceResponse>>> {
        self.batch.push(req)?;
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
        let worker = &self.workers[self.next_worker % self.workers.len()];
        self.next_worker = self.next_worker.wrapping_add(1);
        info!(worker = %worker.name, n = items.len(), "running batch");
        worker.run_batch(&items)
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
    }
}
