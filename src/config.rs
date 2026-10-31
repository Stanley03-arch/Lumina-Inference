use crate::error::{LuminaError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    /// Optional max concurrent in-flight engine submits (0 = unlimited).
    #[serde(default)]
    pub max_in_flight: usize,
    /// Optional request rate limit (tokens/sec). `None` disables.
    #[serde(default)]
    pub rate_limit_per_sec: Option<f64>,
    /// Burst size when rate limiting (defaults to 2× rate).
    #[serde(default)]
    pub rate_limit_burst: Option<f64>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            batch_timeout_ms: 8,
            worker_slots: 2,
            default_model: "default".into(),
            max_in_flight: 0,
            rate_limit_per_sec: None,
            rate_limit_burst: None,
        }
    }
}

impl EngineConfig {
    pub fn validate(&self) -> Result<()> {
        if self.max_batch_size == 0 {
            return Err(LuminaError::Config(
                "max_batch_size must be >= 1".into(),
            ));
        }
        if self.worker_slots == 0 {
            return Err(LuminaError::Config(
                "worker_slots must be >= 1".into(),
            ));
        }
        if let Some(r) = self.rate_limit_per_sec {
            if r <= 0.0 {
                return Err(LuminaError::Config(
                    "rate_limit_per_sec must be > 0 when set".into(),
                ));
            }
        }
        Ok(())
    }

    /// Load from a JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| LuminaError::Config(format!("read config: {e}")))?;
        Self::from_json_str(&data)
    }

    pub fn from_json_str(s: &str) -> Result<Self> {
        let cfg: Self = serde_json::from_str(s)
            .map_err(|e| LuminaError::Config(format!("config json: {e}")))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// `LUMINA_CONFIG` path, else defaults.
    pub fn from_env() -> Result<Self> {
        if let Ok(path) = std::env::var("LUMINA_CONFIG") {
            return Self::from_file(path);
        }
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("LUMINA_MAX_BATCH") {
            if let Ok(n) = v.parse() {
                cfg.max_batch_size = n;
            }
        }
        if let Ok(v) = std::env::var("LUMINA_BATCH_TIMEOUT_MS") {
            if let Ok(n) = v.parse() {
                cfg.batch_timeout_ms = n;
            }
        }
        if let Ok(v) = std::env::var("LUMINA_WORKER_SLOTS") {
            if let Ok(n) = v.parse() {
                cfg.worker_slots = n;
            }
        }
        if let Ok(v) = std::env::var("LUMINA_DEFAULT_MODEL") {
            cfg.default_model = v;
        }
        if let Ok(v) = std::env::var("LUMINA_RATE_LIMIT") {
            if let Ok(n) = v.parse() {
                cfg.rate_limit_per_sec = Some(n);
            }
        }
        cfg.validate()?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let cfg = EngineConfig {
            max_batch_size: 16,
            rate_limit_per_sec: Some(50.0),
            ..Default::default()
        };
        let s = serde_json::to_string(&cfg).unwrap();
        let back = EngineConfig::from_json_str(&s).unwrap();
        assert_eq!(back.max_batch_size, 16);
        assert_eq!(back.rate_limit_per_sec, Some(50.0));
    }
}
