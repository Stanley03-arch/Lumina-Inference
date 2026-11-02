//! Environment variable helpers.

/// Parse truthy env values: `1`, `true`, `yes`, `on` (case-insensitive).
pub fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => {
            let v = v.trim().to_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on" | "y")
        }
        Err(_) => default,
    }
}

pub fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

pub fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

pub fn env_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Comma-separated list; empty → empty vec.
pub fn env_list(key: &str) -> Vec<String> {
    match std::env::var(key) {
        Ok(v) => v
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        assert!(env_bool("LUMINA_TEST_UNSET_BOOL_XYZ", true));
        assert_eq!(env_u64("LUMINA_TEST_UNSET_U64_XYZ", 7), 7);
    }
}
