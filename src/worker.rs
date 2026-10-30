use crate::batch::{InferenceRequest, InferenceResponse};
use crate::error::Result;
use std::time::Instant;

/// Local worker that echoes input (stand-in until ONNX Runtime is integrated).
pub struct LocalWorker {
    pub name: String,
}

impl LocalWorker {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn run_batch(&self, batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>> {
        let start = Instant::now();
        let mut out = Vec::with_capacity(batch.len());
        for req in batch {
            out.push(InferenceResponse {
                id: req.id.clone(),
                model: req.model.clone(),
                // Echo payload — replace with real model forward pass later.
                output: req.input.clone(),
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
        Ok(out)
    }
}
