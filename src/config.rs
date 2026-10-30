use serde::{Deserialize, Serialize};

/// Runtime configuration for the inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Maximum requests coalesced into one batch.
    pub max_batch_size: usize,
    /// Soft wait (ms) before flushing a partial batch.
    pub batch_timeout_ms: u64,
    /// Number of local worker slots (placeholder for distributed workers).
    pub worker_slots: usize,
    /// Default model id when clients omit one.
    pub default_model: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            batch_timeout_ms: 8,
            worker_slots: 2,
            default_model: "default".into(),
        }
    }
}

impl EngineConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.max_batch_size == 0 {
            return Err(crate::LuminaError::Config(
                "max_batch_size must be >= 1".into(),
            ));
        }
        if self.worker_slots == 0 {
            return Err(crate::LuminaError::Config(
                "worker_slots must be >= 1".into(),
            ));
        }
        Ok(())
    }
}
