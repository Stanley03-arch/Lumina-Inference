//! Health probes for remote workers.

use crate::discovery::WorkerNode;
use crate::error::{LuminaError, Result};
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};
use tracing::debug;

/// Result of a single health probe.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub detail: String,
}

/// Parse host:port from an HTTP-style endpoint.
pub fn endpoint_socket_addr(endpoint: &str) -> Result<std::net::SocketAddr> {
    let s = endpoint
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    let addr = s
        .to_socket_addrs()
        .map_err(|e| LuminaError::Config(format!("resolve {s}: {e}")))?
        .next()
        .ok_or_else(|| LuminaError::Config(format!("no address for {s}")))?;
    Ok(addr)
}

/// TCP connect probe (works without gRPC feature).
pub fn probe_tcp(endpoint: &str, timeout: Duration) -> ProbeResult {
    let start = Instant::now();
    let addr = match endpoint_socket_addr(endpoint) {
        Ok(a) => a,
        Err(e) => {
            return ProbeResult {
                ok: false,
                latency_ms: start.elapsed().as_millis() as u64,
                detail: e.to_string(),
            };
        }
    };
    match std::net::TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => ProbeResult {
            ok: true,
            latency_ms: start.elapsed().as_millis() as u64,
            detail: "tcp ok".into(),
        },
        Err(e) => ProbeResult {
            ok: false,
            latency_ms: start.elapsed().as_millis() as u64,
            detail: format!("tcp: {e}"),
        },
    }
}

/// Probe a worker node over TCP.
pub fn probe_node(node: &WorkerNode, timeout: Duration) -> ProbeResult {
    let r = probe_tcp(&node.endpoint, timeout);
    debug!(id = %node.id, ok = r.ok, ms = r.latency_ms, "health probe");
    r
}

/// gRPC Health RPC when feature is on.
#[cfg(feature = "grpc")]
pub async fn probe_grpc(endpoint: &str) -> ProbeResult {
    let start = Instant::now();
    match crate::grpc_client::InferenceClient::connect(endpoint).await {
        Ok(mut client) => match client.health().await {
            Ok((ok, version)) => ProbeResult {
                ok,
                latency_ms: start.elapsed().as_millis() as u64,
                detail: version,
            },
            Err(e) => ProbeResult {
                ok: false,
                latency_ms: start.elapsed().as_millis() as u64,
                detail: e.to_string(),
            },
        },
        Err(e) => ProbeResult {
            ok: false,
            latency_ms: start.elapsed().as_millis() as u64,
            detail: e.to_string(),
        },
    }
}

#[cfg(not(feature = "grpc"))]
pub async fn probe_grpc(endpoint: &str) -> ProbeResult {
    probe_tcp(endpoint, Duration::from_secs(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_endpoint() {
        let a = endpoint_socket_addr("http://127.0.0.1:50051").unwrap();
        assert_eq!(a.port(), 50051);
    }
}
