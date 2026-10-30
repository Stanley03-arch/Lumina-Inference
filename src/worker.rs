use crate::batch::{InferenceRequest, InferenceResponse};
use crate::error::Result;
use crate::tensor::NamedTensors;
use std::time::Instant;

/// Pluggable inference backend (echo, ONNX, remote, …).
pub trait InferenceWorker: Send + Sync {
    fn name(&self) -> &str;
    fn run_batch(&self, batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>>;
}

/// Local worker that echoes input (default backend / tests).
pub struct LocalWorker {
    name: String,
}

impl LocalWorker {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl InferenceWorker for LocalWorker {
    fn name(&self) -> &str {
        &self.name
    }

    fn run_batch(&self, batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>> {
        let start = Instant::now();
        let mut out = Vec::with_capacity(batch.len());
        for req in batch {
            out.push(InferenceResponse {
                id: req.id.clone(),
                model: req.model.clone(),
                output: req.input.clone(),
                latency_ms: start.elapsed().as_millis() as u64,
                named_outputs: NamedTensors::default(),
            });
        }
        Ok(out)
    }
}
