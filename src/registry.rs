use crate::error::{LuminaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub version: String,
    pub path: Option<String>,
    pub loaded: bool,
}

/// In-memory model catalog (paths only until ONNX load lands).
#[derive(Debug, Default)]
pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, info: ModelInfo) {
        self.models.insert(info.id.clone(), info);
    }

    pub fn get(&self, id: &str) -> Result<&ModelInfo> {
        self.models
            .get(id)
            .ok_or_else(|| LuminaError::ModelNotFound(id.to_string()))
    }

    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    pub fn mark_loaded(&mut self, id: &str, loaded: bool) -> Result<()> {
        let m = self
            .models
            .get_mut(id)
            .ok_or_else(|| LuminaError::ModelNotFound(id.to_string()))?;
        m.loaded = loaded;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get() {
        let mut reg = ModelRegistry::new();
        reg.register(ModelInfo {
            id: "tiny".into(),
            version: "1".into(),
            path: Some("models/tiny.onnx".into()),
            loaded: false,
        });
        assert_eq!(reg.get("tiny").unwrap().version, "1");
        assert!(reg.get("missing").is_err());
    }
}
