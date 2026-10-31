use crate::config::EngineConfig;
use crate::error::{LuminaError, Result};
use crate::tensor::NamedTensors;
use serde::{Deserialize, Serialize};
use std::time::Instant;

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
    /// Soft deadline from request start (milliseconds). `None` = no deadline.
    #[serde(default)]
    pub deadline_ms: Option<u64>,
    /// Optional sticky affinity key (client / session id).
    #[serde(default)]
    pub affinity_key: Option<String>,
}

impl InferenceRequest {
    pub fn new(id: impl Into<String>, model: impl Into<String>, input: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            model: model.into(),
            input,
            named_inputs: NamedTensors::default(),
            deadline_ms: None,
            affinity_key: None,
        }
    }

    pub fn with_deadline_ms(mut self, ms: u64) -> Self {
        self.deadline_ms = Some(ms);
        self
    }

    pub fn with_affinity(mut self, key: impl Into<String>) -> Self {
        self.affinity_key = Some(key.into());
        self
    }

    /// Returns `Err(DeadlineExceeded)` if `started` is past the request deadline.
    pub fn check_deadline(&self, started: Instant) -> Result<()> {
        if let Some(ms) = self.deadline_ms {
            if started.elapsed().as_millis() as u64 >= ms {
                return Err(LuminaError::DeadlineExceeded);
            }
        }
        Ok(())
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
    use std::time::Duration;

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

    #[test]
    fn deadline_fires() {
        let req = InferenceRequest::new("1", "m", vec![]).with_deadline_ms(1);
        let start = Instant::now() - Duration::from_millis(5);
        assert!(matches!(
            req.check_deadline(start),
            Err(LuminaError::DeadlineExceeded)
        ));
    }
}
