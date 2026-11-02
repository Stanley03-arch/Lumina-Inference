//! Readiness / liveness check aggregation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

/// Named readiness checks; overall OK only if all pass (or required ones).
#[derive(Debug, Clone, Default)]
pub struct Readiness {
    results: HashMap<String, CheckResult>,
}

impl Readiness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: impl Into<String>, ok: bool, detail: impl Into<String>) {
        let name = name.into();
        self.results.insert(
            name.clone(),
            CheckResult {
                name,
                ok,
                detail: detail.into(),
            },
        );
    }

    pub fn remove(&mut self, name: &str) {
        self.results.remove(name);
    }

    pub fn is_ready(&self) -> bool {
        !self.results.is_empty() && self.results.values().all(|c| c.ok)
    }

    /// Ready if every name in `required` is present and ok.
    pub fn is_ready_required(&self, required: &[&str]) -> bool {
        required.iter().all(|n| {
            self.results
                .get(*n)
                .map(|c| c.ok)
                .unwrap_or(false)
        })
    }

    pub fn checks(&self) -> Vec<CheckResult> {
        self.results.values().cloned().collect()
    }

    pub fn to_json(&self) -> String {
        let body = serde_json::json!({
            "ready": self.is_ready(),
            "checks": self.checks(),
        });
        body.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_must_pass() {
        let mut r = Readiness::new();
        r.set("workers", true, "3 up");
        r.set("model", false, "missing");
        assert!(!r.is_ready());
        r.set("model", true, "loaded");
        assert!(r.is_ready());
    }
}
