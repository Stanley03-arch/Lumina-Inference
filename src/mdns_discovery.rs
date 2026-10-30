//! mDNS / DNS-SD discovery for Lumina workers (`--features mdns`).
//!
//! Service type: `_lumina._tcp.local.`
//! TXT keys: `id`, `endpoint`, `models`, `capacity`

use crate::discovery::{WorkerNode, WorkerRegistry};
use crate::error::{LuminaError, Result};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use tracing::{debug, info, warn};

/// DNS-SD service type for Lumina inference workers.
pub const SERVICE_TYPE: &str = "_lumina._tcp.local.";

/// RAII handle that keeps the mDNS daemon alive while registered.
pub struct MdnsAdvertiser {
    #[cfg(feature = "mdns")]
    daemon: Option<mdns_sd::ServiceDaemon>,
    #[cfg(not(feature = "mdns"))]
    _priv: (),
}

impl MdnsAdvertiser {
    /// Stop advertising (also runs on drop when feature is on).
    pub fn shutdown(self) {
        #[cfg(feature = "mdns")]
        if let Some(d) = self.daemon {
            let _ = d.shutdown();
        }
    }
}

#[cfg(feature = "mdns")]
impl Drop for MdnsAdvertiser {
    fn drop(&mut self) {
        if let Some(d) = self.daemon.take() {
            let _ = d.shutdown();
        }
    }
}

/// Advertise `node` on the local link via mDNS.
///
/// Keep the returned [`MdnsAdvertiser`] alive for as long as the service should be visible.
pub fn advertise(node: &WorkerNode, grpc_port: u16) -> Result<MdnsAdvertiser> {
    #[cfg(not(feature = "mdns"))]
    {
        let _ = (node, grpc_port);
        return Err(LuminaError::Config(
            "mDNS support not compiled; rebuild with `--features mdns`".into(),
        ));
    }

    #[cfg(feature = "mdns")]
    {
        let daemon = mdns_sd::ServiceDaemon::new()
            .map_err(|e| LuminaError::Internal(format!("mdns daemon: {e}")))?;

        let host = format!("{}.local.", sanitize_host(&node.id));
        let models = node.models.join(",");
        let cap = node.capacity.to_string();
        let props = [
            ("id", node.id.as_str()),
            ("endpoint", node.endpoint.as_str()),
            ("models", models.as_str()),
            ("capacity", cap.as_str()),
        ];

        let info = if let Some(ip) = endpoint_ip(&node.endpoint) {
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

        daemon
            .register(info)
            .map_err(|e| LuminaError::Internal(format!("mdns register: {e}")))?;

        info!(
            id = %node.id,
            service = SERVICE_TYPE,
            port = grpc_port,
            "mDNS service registered"
        );
        Ok(MdnsAdvertiser {
            daemon: Some(daemon),
        })
    }
}

/// Browse for Lumina workers and merge resolved peers into `registry` until `timeout`.
pub fn browse_into_registry(registry: &mut WorkerRegistry, timeout: Duration) -> Result<usize> {
    #[cfg(not(feature = "mdns"))]
    {
        let _ = (registry, timeout);
        return Err(LuminaError::Config(
            "mDNS support not compiled; rebuild with `--features mdns`".into(),
        ));
    }

    #[cfg(feature = "mdns")]
    {
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
}

/// Background browse loop updating a shared registry.
pub fn spawn_mdns_browser(
    registry: crate::gossip::SharedRegistry,
    poll: Duration,
) -> std::thread::JoinHandle<()> {
    #[cfg(not(feature = "mdns"))]
    {
        let _ = (registry, poll);
        return std::thread::spawn(|| {
            warn!("spawn_mdns_browser called without `--features mdns`");
        });
    }

    #[cfg(feature = "mdns")]
    {
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
            let _ = daemon.shutdown();
        })
    }
}

#[cfg(feature = "mdns")]
fn node_from_resolved(info: &mdns_sd::ServiceInfo) -> Option<WorkerNode> {
    let mut props: HashMap<String, String> = HashMap::new();
    for p in info.get_properties().iter() {
        props.insert(p.key().to_string(), p.val_str().to_string());
    }

    let id = props
        .get("id")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            info.get_fullname()
                .split('.')
                .next()
                .unwrap_or("unknown")
                .to_string()
        });

    let endpoint = if let Some(ep) = props.get("endpoint").filter(|s| !s.is_empty()) {
        ep.clone()
    } else {
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
        .filter(|v: &Vec<String>| !v.is_empty())
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
        last_seen: 0,
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
