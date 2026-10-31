//! Request trace / correlation IDs.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique-ish trace id: `lum-<ms>-<counter>`.
pub fn new_trace_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("lum-{ms}-{n}")
}

/// Prefer an incoming header value; otherwise allocate a new id.
pub fn resolve_trace_id(incoming: Option<&str>) -> String {
    match incoming.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => sanitize(s),
        None => new_trace_id(),
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .take(128)
        .collect()
}

/// Header names commonly used for propagation.
pub const HEADER_TRACE: &str = "x-request-id";
pub const HEADER_TRACE_ALT: &str = "x-lumina-trace-id";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_unique() {
        let a = new_trace_id();
        let b = new_trace_id();
        assert_ne!(a, b);
        assert!(a.starts_with("lum-"));
    }

    #[test]
    fn resolves_incoming() {
        assert_eq!(resolve_trace_id(Some("abc-123")), "abc-123");
        assert!(resolve_trace_id(None).starts_with("lum-"));
    }
}
