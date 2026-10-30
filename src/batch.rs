use crate::config::EngineConfig;
use crate::error::{LuminaError, Result};
use crate::tensor::NamedTensors;
use serde::{Deserialize, Serialize};

/// Single inference request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: String,
    pub model: String,
    /// Primary / default input bytes (used when `named_inputs` is empty).
    pub input: Vec<u8>,
    /// Optional named tensors for multi-input ONNX models.
    #[serde(default)]
    pub named_inputs: NamedTensors,
}

impl InferenceRequest {
    pub fn new(id: impl Into<String>, model: impl Into<String>, input: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            model: model.into(),
            input,
            named_inputs: NamedTensors::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub id: String,
    pub model: String,
    pub output: Vec<u8>,
    pub latency_ms: u64,
    /// Optional named outputs when the worker fills them.
    #[serde(default)]
    pub named_outputs: NamedTensors,
}

impl InferenceResponse {
    pub fn new(id: String, model: String, output: Vec<u8>, latency_ms: u64) -> Self {
        Self {
            id,
            model,
            output,
            latency_ms,
            named_outputs: NamedTensors::default(),
        }
    }
}

/// Accumulates requests until size or timeout policy is met.
#[derive(Debug, Default)]
pub struct BatchAccumulator {
    items: Vec<InferenceRequest>,
    max: usize,
}

impl BatchAccumulator {
    pub fn new(cfg: &EngineConfig) -> Self {
        Self {
            items: Vec::with_capacity(cfg.max_batch_size),
            max: cfg.max_batch_size,
        }
    }

    pub fn push(&mut self, req: InferenceRequest) -> Result<()> {
        if self.items.len() >= self.max {
            return Err(LuminaError::BatchOverflow { max: self.max });
        }
        self.items.push(req);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.max
    }

    pub fn take(&mut self) -> Vec<InferenceRequest> {
        std::mem::take(&mut self.items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_to_max() {
        let cfg = EngineConfig {
            max_batch_size: 2,
            ..Default::default()
        };
        let mut b = BatchAccumulator::new(&cfg);
        b.push(InferenceRequest::new("1", "m", vec![1])).unwrap();
        b.push(InferenceRequest::new("2", "m", vec![2])).unwrap();
        assert!(b.is_full());
        assert!(b.push(InferenceRequest::new("3", "m", vec![3])).is_err());
    }
}
