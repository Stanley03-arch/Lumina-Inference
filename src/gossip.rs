//! UDP gossip for dynamic worker discovery.
//!
//! Nodes periodically broadcast a JSON [`GossipMessage`] on a shared
//! multicast or broadcast address. Peers listen and merge into a
//! [`WorkerRegistry`]. This is a lightweight LAN-oriented alternative
//! to static peer lists — not a full membership protocol.

use crate::discovery::{WorkerNode, WorkerRegistry};
use crate::error::{LuminaError, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Default gossip port (UDP).
pub const DEFAULT_GOSSIP_PORT: u16 = 19551;

/// Wire message (JSON over UDP).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub node: WorkerNode,
    /// Protocol version.
    pub v: u32,
}

impl GossipMessage {
    pub fn announce(node: WorkerNode) -> Self {
        Self { node, v: 1 }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| LuminaError::Internal(format!("gossip encode: {e}")))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| LuminaError::Internal(format!("gossip decode: {e}")))
    }
}

/// Gossip runtime configuration.
#[derive(Debug, Clone)]
pub struct GossipConfig {
    /// Local bind address, e.g. `0.0.0.0:19551`.
    pub bind: SocketAddr,
    /// Where to send announces (broadcast / multicast / unicast peer).
    pub announce_to: SocketAddr,
    /// How often to broadcast our own node.
    pub announce_interval: Duration,
    /// Skip messages from this node id (self).
    pub self_id: Option<String>,
}

impl GossipConfig {
    pub fn lan(port: u16) -> Self {
        Self {
            bind: format!("0.0.0.0:{port}").parse().expect("bind"),
            // Limited broadcast — works on many LANs without multicast setup.
            announce_to: format!("255.255.255.255:{port}").parse().expect("dest"),
            announce_interval: Duration::from_secs(5),
            self_id: None,
        }
    }

    pub fn from_env() -> Result<Option<Self>> {
        let enabled = std::env::var("LUMINA_GOSSIP")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if !enabled {
            return Ok(None);
        }
        let port: u16 = std::env::var("LUMINA_GOSSIP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_GOSSIP_PORT);
        let mut cfg = Self::lan(port);
        if let Ok(dest) = std::env::var("LUMINA_GOSSIP_ADDR") {
            cfg.announce_to = dest
                .parse()
                .map_err(|e| LuminaError::Config(format!("LUMINA_GOSSIP_ADDR: {e}")))?;
        }
        if let Ok(bind) = std::env::var("LUMINA_GOSSIP_BIND") {
            cfg.bind = bind
                .parse()
                .map_err(|e| LuminaError::Config(format!("LUMINA_GOSSIP_BIND: {e}")))?;
        }
        if let Ok(secs) = std::env::var("LUMINA_GOSSIP_INTERVAL_SECS") {
            if let Ok(s) = secs.parse::<u64>() {
                cfg.announce_interval = Duration::from_secs(s.max(1));
            }
        }
        Ok(Some(cfg))
    }
}

/// Shared registry updated by the gossip listener.
pub type SharedRegistry = Arc<Mutex<WorkerRegistry>>;

/// Run announce loop: periodically broadcast `local` node.
pub async fn run_announcer(
    cfg: GossipConfig,
    local: WorkerNode,
) -> Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| LuminaError::Internal(format!("gossip announce bind: {e}")))?;
    sock.set_broadcast(true)
        .map_err(|e| LuminaError::Internal(format!("broadcast: {e}")))?;

    info!(
        to = %cfg.announce_to,
        id = %local.id,
        interval_secs = cfg.announce_interval.as_secs(),
        "gossip announcer started"
    );

    let mut tick = tokio::time::interval(cfg.announce_interval);
    loop {
        tick.tick().await;
        let mut node = local.clone();
        node.touch();
        let msg = GossipMessage::announce(node);
        match msg.encode() {
            Ok(bytes) => {
                if let Err(e) = sock.send_to(&bytes, cfg.announce_to).await {
                    warn!(error = %e, "gossip announce send failed");
                } else {
                    debug!(id = %local.id, "gossip announced");
                }
            }
            Err(e) => warn!(error = %e, "gossip encode failed"),
        }
    }
}

/// Listen for peer announces and merge into `registry`.
pub async fn run_listener(cfg: GossipConfig, registry: SharedRegistry) -> Result<()> {
    let sock = UdpSocket::bind(cfg.bind)
        .await
        .map_err(|e| LuminaError::Internal(format!("gossip listen bind {}: {e}", cfg.bind)))?;
    sock.set_broadcast(true).ok();

    info!(%cfg.bind, "gossip listener started");
    let mut buf = vec![0u8; 65535];

    loop {
        let (n, from) = sock
            .recv_from(&mut buf)
            .await
            .map_err(|e| LuminaError::Internal(format!("gossip recv: {e}")))?;
        match GossipMessage::decode(&buf[..n]) {
            Ok(msg) => {
                if msg.v != 1 {
                    debug!(from = %from, v = msg.v, "skip unsupported gossip version");
                    continue;
                }
                if let Some(ref self_id) = cfg.self_id {
                    if &msg.node.id == self_id {
                        continue;
                    }
                }
                let mut reg = registry.lock().await;
                reg.register(msg.node);
                reg.reap();
            }
            Err(e) => {
                debug!(from = %from, error = %e, "bad gossip packet");
            }
        }
    }
}

/// Spawn announcer + listener as background tasks.
/// Returns the shared registry (pre-loaded if you pass one in).
pub async fn spawn_gossip(
    mut cfg: GossipConfig,
    local: WorkerNode,
    mut registry: WorkerRegistry,
) -> Result<(SharedRegistry, Vec<tokio::task::JoinHandle<()>>)> {
    cfg.self_id = Some(local.id.clone());
    registry.register(local.clone());
    let shared: SharedRegistry = Arc::new(Mutex::new(registry));

    let ann_cfg = cfg.clone();
    let ann_node = local.clone();
    let h1 = tokio::spawn(async move {
        if let Err(e) = run_announcer(ann_cfg, ann_node).await {
            warn!(error = %e, "gossip announcer exited");
        }
    });

    let listen_cfg = cfg;
    let reg2 = Arc::clone(&shared);
    let h2 = tokio::spawn(async move {
        if let Err(e) = run_listener(listen_cfg, reg2).await {
            warn!(error = %e, "gossip listener exited");
        }
    });

    Ok((shared, vec![h1, h2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_message() {
        let n = WorkerNode::new("w1", "http://10.0.0.1:50051");
        let msg = GossipMessage::announce(n.clone());
        let bytes = msg.encode().unwrap();
        let back = GossipMessage::decode(&bytes).unwrap();
        assert_eq!(back.node.id, "w1");
        assert_eq!(back.v, 1);
    }
}
