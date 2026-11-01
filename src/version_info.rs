//! Build / package version helpers.

/// Cargo package version (`CARGO_PKG_VERSION`).
pub fn package_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Human-readable summary of compile-time features.
pub fn feature_summary() -> String {
    let mut parts = vec!["core".to_string()];
    #[cfg(feature = "onnx")]
    parts.push("onnx".into());
    #[cfg(feature = "grpc")]
    parts.push("grpc".into());
    #[cfg(feature = "mdns")]
    parts.push("mdns".into());
    format!("lumina-inference {} [{}]", package_version(), parts.join("+"))
}

/// Structured version payload for `/version` style endpoints.
pub fn version_json() -> String {
    format!(
        "{{\n  \"name\": \"lumina-inference\",\n  \"version\": \"{}\",\n  \"features\": {{\n    \"onnx\": {},\n    \"grpc\": {},\n    \"mdns\": {}\n  }}\n}}",
        package_version(),
        cfg!(feature = "onnx"),
        cfg!(feature = "grpc"),
        cfg!(feature = "mdns"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_nonempty() {
        assert!(!package_version().is_empty());
        assert!(feature_summary().contains("lumina-inference"));
        assert!(version_json().contains("version"));
    }
}
