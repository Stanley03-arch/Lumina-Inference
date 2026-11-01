//! Host / process identity helpers for worker registration.

use std::sync::OnceLock;

static HOST: OnceLock<String> = OnceLock::new();

/// Best-effort hostname (`HOSTNAME` env, else `localhost`).
pub fn hostname() -> &'static str {
    HOST.get_or_init(|| {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "localhost".into())
    })
    .as_str()
}

/// Process id as string.
pub fn pid_string() -> String {
    std::process::id().to_string()
}

/// Suggested worker id: `{hostname}-{pid}` or override via `LUMINA_WORKER_ID`.
pub fn default_worker_id() -> String {
    if let Ok(id) = std::env::var("LUMINA_WORKER_ID") {
        if !id.trim().is_empty() {
            return id;
        }
    }
    format!("{}-{}", hostname(), std::process::id())
}

/// Suggested gRPC advertise endpoint from env or localhost.
pub fn default_endpoint(port: u16) -> String {
    if let Ok(ep) = std::env::var("LUMINA_ADVERTISE_ENDPOINT") {
        if !ep.trim().is_empty() {
            return ep;
        }
    }
    format!("http://{}:{port}", hostname())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_id_nonempty() {
        assert!(!default_worker_id().is_empty());
        assert!(!hostname().is_empty());
    }
}
