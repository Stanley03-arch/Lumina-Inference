//! Guardrails on request/response payload sizes.

use crate::batch::InferenceRequest;
use crate::error::{LuminaError, Result};

/// Limits on byte sizes to avoid OOM from huge payloads.
#[derive(Debug, Clone)]
pub struct MemoryGuard {
    pub max_input_bytes: usize,
    pub max_named_tensor_bytes: usize,
    pub max_named_tensors: usize,
}

impl Default for MemoryGuard {
    fn default() -> Self {
        Self {
            max_input_bytes: 16 * 1024 * 1024, // 16 MiB
            max_named_tensor_bytes: 64 * 1024 * 1024,
            max_named_tensors: 32,
        }
    }
}

impl MemoryGuard {
    pub fn check_request(&self, req: &InferenceRequest) -> Result<()> {
        if req.input.len() > self.max_input_bytes {
            return Err(LuminaError::Config(format!(
                "input {} bytes exceeds max {}",
                req.input.len(),
                self.max_input_bytes
            )));
        }
        if req.named_inputs.tensors.len() > self.max_named_tensors {
            return Err(LuminaError::Config(format!(
                "too many named tensors: {} > {}",
                req.named_inputs.tensors.len(),
                self.max_named_tensors
            )));
        }
        let mut total = 0usize;
        for (_, data) in &req.named_inputs.tensors {
            total = total.saturating_add(data.len());
            if total > self.max_named_tensor_bytes {
                return Err(LuminaError::Config(format!(
                    "named tensor payload {} exceeds max {}",
                    total, self.max_named_tensor_bytes
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_huge_input() {
        let g = MemoryGuard {
            max_input_bytes: 8,
            ..Default::default()
        };
        let req = InferenceRequest::new("1", "m", vec![0u8; 16]);
        assert!(g.check_request(&req).is_err());
    }
}
