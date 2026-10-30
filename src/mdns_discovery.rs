//! mDNS / DNS-SD discovery for Lumina workers (`--features mdns`).
//!
//! Service type: `_lumina._tcp.local.`
//! TXT keys: `id`, `endpoint`, `models`, `capacity`

use crate::discovery::{WorkerNode, WorkerRegistry};
use crate::error::{LuminaError, Result};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// DNS-SD service type for Lumina inference workers.
pub const SERVICE_TYPE: &str = "_lumina._tcp.local.";

/// Advertise `node` on the local link via mDNS and keep the daemon alive.
///
/// `grpc_port` is the TCP port published in the SRV record (parsed from endpoint if possible).
#[cfg(feature = "mdns")]
pub fn advertise(node: &WorkerNode, grpc_port: u16) -> Result<mdns_sd::ServiceDaemon> {
    let daemon = mdns_sd::ServiceDaemon::new()
        .map_err(|e| LuminaError::Internal(format!("mdns daemon: {e}")))?;

    let host = format!("{}.local.", sanitize_host(&node.id));
    let models = node.models.join(",");
    let props = [
        ("id", node.id.as_str()),
        ("endpoint", node.endpoint.as_str()),
        ("models", models.as_str()),
        ("capacity", &*node.capacity.to_string()),
    ];

    // Prefer IP from endpoint if present; else let mdns-sd pick interfaces.
    let ip_opt = endpoint_ip(&node.endpoint);
    let mut info = if let Some(ip) = ip_opt {
        mdns_sd::ServiceInfo::new(
            SERVICE_TYPE,
            &node.id,
            &host,
            ip,
            grpc_port,
            &props[..],
        )
        .map_err(|e| LuminaError::Internal(format!("mdns ServiceInfo: {e}")))?
    } else {
        mdns_sd::ServiceInfo::new(
            SERVICE_TYPE,
            &node.id,
            &host,
            "",
            grpc_port,
            &props[..],
        )
        .map_err(|e| LuminaError::Internal(format!("mdns ServiceInfo: {e}")))?
        .enable_addr_auto()
    };

    // Silence unused mut warning on some mdns-sd versions
    let _ = &mut info;

    daemon
        .register(info)
        .map_err(|e| LuminaError::Internal(format!("mdns register: {e}")))?;

    info!(
        id = %node.id,
        service = SERVICE_TYPE,
        port = grpc_port,
        "mDNS service registered"
    );
    Ok(daemon)
}

/// Browse for Lumina workers and merge resolved peers into `registry` until `timeout`.
#[cfg(feature = "mdns")]
pub fn browse_into_registry(
    registry: &mut WorkerRegistry,
    timeout: Duration,
) -> Result<usize> {
    let daemon = mdns_sd::ServiceDaemon::new()
        .map_err(|e| LuminaError::Internal(format!("mdns daemon: {e}")))?;
    let receiver = daemon
        .browse(SERVICE_TYPE)
        .map_err(|e| LuminaError::Internal(format!("mdns browse: {e}")))?;

    let deadline = std::time::Instant::now() + timeout;
    let mut found = 0usize;

    while std::time::Instant::now() < deadline {
        let wait = deadline.saturating_duration_since(std::time::Instant::now());
        match receiver.recv_timeout(wait) {
            Ok(event) => match event {
                mdns_sd::ServiceEvent::ServiceResolved(info) => {
                    if let Some(node) = node_from_resolved(&info) {
                        debug!(id = %node.id, endpoint = %node.endpoint, "mDNS resolved");
                        registry.register(node);
                        found += 1;
                    }
                }
                mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                    // Instance name is the first label of the fullname.
                    let id = fullname.split('.').next().unwrap_or("");
                    if !id.is_empty() {
                        registry.deregister(id);
                        debug!(%id, "mDNS service removed");
                    }
                }
                other => {
                    debug!(?other, "mDNS event");
                }
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = daemon.shutdown();
    info!(found, "mDNS browse finished");
    Ok(found)
}

/// Background browse loop updating a shared registry.
#[cfg(feature = "mdns")]
pub fn spawn_mdns_browser(
    registry: crate::gossip::SharedRegistry,
    poll: Duration,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let daemon = match mdns_sd::ServiceDaemon::new() {
            Ok(d) => d,
            Err(e) => {
                warn!(error = %e, "mdns browser daemon failed");
                return;
            }
        };
        let receiver = match daemon.browse(SERVICE_TYPE) {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "mdns browse failed");
                return;
            }
        };
        info!(service = SERVICE_TYPE, "mDNS browser started");
        loop {
            match receiver.recv_timeout(poll) {
                Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                    if let Some(node) = node_from_resolved(&info) {
                        let mut reg = registry.blocking_lock();
                        reg.register(node);
                        reg.reap();
                    }
                }
                Ok(mdns_sd::ServiceEvent::ServiceRemoved(_, fullname)) => {
                    let id = fullname.split('.').next().unwrap_or("");
                    if !id.is_empty() {
                        let mut reg = registry.blocking_lock();
                        reg.deregister(id);
                    }
                }
                Ok(_) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let mut reg = registry.blocking_lock();
                    reg.reap();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    })
}

#[cfg(feature = "mdns")]
fn node_from_resolved(info: &mdns_sd::ServiceInfo) -> Option<WorkerNode> {
    let props: HashMap<String, String> = info
        .get_properties()
        .iter()
        .map(|p| (p.key().to_string(), p.val_str().to_string()))
        .collect();

    let id = props
        .get("id")
        .cloned()
        .unwrap_or_else(|| info.get_hostname().trim_end_matches('.').to_string());

    let endpoint = if let Some(ep) = props.get("endpoint") {
        ep.clone()
    } else {
        // Build from addresses + port
        let port = info.get_port();
        let addr = info.get_addresses().iter().next()?;
        format!("http://{addr}:{port}")
    };

    let models: Vec<String> = props
        .get("models")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_else(|| vec!["default".into()]);

    let capacity = props
        .get("capacity")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    Some(WorkerNode {
        id,
        endpoint,
        models,
        capacity,
        last_seen: 0, // register() will touch
    })
}

fn sanitize_host(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn endpoint_ip(endpoint: &str) -> Option<IpAddr> {
    // http://host:port or host:port
    let s = endpoint
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host = s.split(':').next()?;
    host.parse().ok()
}

/// Parse gRPC port from endpoint URL (`http://host:50051` → 50051).
pub fn port_from_endpoint(endpoint: &str) -> u16 {
    let s = endpoint
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    s.rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50051)
}

/// Stubs when `mdns` feature is off.
#[cfg(not(feature = "mdns"))]
pub fn advertise(_node: &WorkerNode, _grpc_port: u16) -> Result<()> {
    Err(LuminaError::Config(
        "mDNS support not compiled; rebuild with `--features mdns`".into(),
    ))
}

#[cfg(not(feature = "mdns"))]
pub fn browse_into_registry(
    _registry: &mut WorkerRegistry,
    _timeout: Duration,
) -> Result<usize> {
    Err(LuminaError::Config(
        "mDNS support not compiled; rebuild with `--features mdns`".into(),
    ))
}

#[cfg(not(feature = "mdns"))]
pub fn spawn_mdns_browser(
    _registry: crate::gossip::SharedRegistry,
    _poll: Duration,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        warn!("spawn_mdns_browser called without `--features mdns`");
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_parse() {
        assert_eq!(port_from_endpoint("http://10.0.0.1:50051"), 50051);
        assert_eq!(port_from_endpoint("http://localhost:9"), 9);
    }

    #[test]
    fn sanitize() {
        assert_eq!(sanitize_host("GPU_A"), "gpu-a");
    }
}
