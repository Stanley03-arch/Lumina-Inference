//! Runtime feature flags (boolean + percentage rollout).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    pub enabled: bool,
    /// 0–100; only consulted when `enabled` is true.
    #[serde(default = "hundred")]
    pub percent: u8,
}

fn hundred() -> u8 {
    100
}

impl Flag {
    pub fn on() -> Self {
        Self {
            enabled: true,
            percent: 100,
        }
    }

    pub fn off() -> Self {
        Self {
            enabled: false,
            percent: 0,
        }
    }

    pub fn percent(p: u8) -> Self {
        Self {
            enabled: true,
            percent: p.min(100),
        }
    }
}

/// Named feature flags evaluated per key (e.g. user / request id).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    flags: HashMap<String, Flag>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: impl Into<String>, flag: Flag) {
        self.flags.insert(name.into(), flag);
    }

    pub fn get(&self, name: &str) -> Option<&Flag> {
        self.flags.get(name)
    }

    /// Deterministic rollout: hash(`name` + `key`) % 100 < percent.
    pub fn is_enabled(&self, name: &str, key: &str) -> bool {
        let Some(f) = self.flags.get(name) else {
            return false;
        };
        if !f.enabled {
            return false;
        }
        if f.percent >= 100 {
            return true;
        }
        if f.percent == 0 {
            return false;
        }
        let h = simple_hash(name, key);
        (h % 100) < f.percent as u64
    }

    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

fn simple_hash(name: &str, key: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes().chain(b"\0".iter().copied()).chain(key.bytes()) {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_rollout_stable() {
        let mut ff = FeatureFlags::new();
        ff.set("new_batcher", Flag::percent(50));
        let a = ff.is_enabled("new_batcher", "user-1");
        let b = ff.is_enabled("new_batcher", "user-1");
        assert_eq!(a, b);
        assert!(!ff.is_enabled("missing", "x"));
    }
}
