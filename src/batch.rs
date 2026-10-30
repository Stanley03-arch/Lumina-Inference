use crate::config::EngineConfig;
use crate::error::{LuminaError, Result};
use serde::{Deserialize, Serialize};

/// Single inference request (placeholder tensor payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: String,
    pub model: String,
    /// Opaque input bytes until ONNX tensors are wired in.
    pub input: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub id: String,
    pub model: String,
    pub output: Vec<u8>,
    pub latency_ms: u64,
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
        b.push(InferenceRequest {
            id: "1".into(),
            model: "m".into(),
            input: vec![1],
        })
        .unwrap();
        b.push(InferenceRequest {
            id: "2".into(),
            model: "m".into(),
            input: vec![2],
        })
        .unwrap();
        assert!(b.is_full());
        assert!(b
            .push(InferenceRequest {
                id: "3".into(),
                model: "m".into(),
                input: vec![3],
            })
            .is_err());
    }
}
