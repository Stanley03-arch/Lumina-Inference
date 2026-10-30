//! Multi-node worker discovery.
//!
//! Nodes register themselves (or are listed statically). A coordinator
//! picks healthy peers for remote inference. Heartbeats mark liveness.

use crate::error::{LuminaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A discovered inference worker node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerNode {
    pub id: String,
    /// gRPC endpoint, e.g. `http://10.0.0.2:50051`.
    pub endpoint: String,
    /// Models this node can serve.
    #[serde(default)]
    pub models: Vec<String>,
    /// Soft capacity hint (concurrent slots).
    #[serde(default = "default_capacity")]
    pub capacity: u32,
    /// Unix seconds of last successful heartbeat / registration.
    #[serde(default)]
    pub last_seen: u64,
}

fn default_capacity() -> u32 {
    4
}

impl WorkerNode {
    pub fn new(id: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            endpoint: endpoint.into(),
            models: vec!["default".into()],
            capacity: 4,
            last_seen: now_secs(),
        }
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = models;
        self
    }

    pub fn touch(&mut self) {
        self.last_seen = now_secs();
    }

    pub fn is_fresh(&self, ttl: Duration) -> bool {
        let age = now_secs().saturating_sub(self.last_seen);
        age <= ttl.as_secs()
    }

    pub fn serves(&self, model: &str) -> bool {
        self.models.iter().any(|m| m == model || m == "*")
    }
}

/// In-memory registry of worker nodes.
#[derive(Debug, Default, Clone)]
pub struct WorkerRegistry {
    nodes: HashMap<String, WorkerNode>,
    /// Max age before a node is considered dead.
    pub ttl: Duration,
    rr: usize,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            ttl: Duration::from_secs(30),
            rr: 0,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn register(&mut self, mut node: WorkerNode) {
        node.touch();
        info!(id = %node.id, endpoint = %node.endpoint, "worker registered");
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn heartbeat(&mut self, id: &str) -> Result<()> {
        let n = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| LuminaError::WorkerUnavailable(id.into()))?;
        n.touch();
        debug!(%id, "heartbeat");
        Ok(())
    }

    pub fn deregister(&mut self, id: &str) -> Option<WorkerNode> {
        let n = self.nodes.remove(id);
        if n.is_some() {
            info!(%id, "worker deregistered");
        }
        n
    }

    pub fn get(&self, id: &str) -> Option<&WorkerNode> {
        self.nodes.get(id)
    }

    /// All nodes still within TTL.
    pub fn healthy(&self) -> Vec<&WorkerNode> {
        self.nodes
            .values()
            .filter(|n| n.is_fresh(self.ttl))
            .collect()
    }

    /// Healthy nodes that advertise `model`.
    pub fn healthy_for_model(&self, model: &str) -> Vec<&WorkerNode> {
        self.healthy()
            .into_iter()
            .filter(|n| n.serves(model))
            .collect()
    }

    /// Round-robin pick among healthy nodes for `model`.
    pub fn pick(&mut self, model: &str) -> Option<WorkerNode> {
        let candidates: Vec<WorkerNode> = self
            .healthy_for_model(model)
            .into_iter()
            .cloned()
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let idx = self.rr % candidates.len();
        self.rr = self.rr.wrapping_add(1);
        Some(candidates[idx].clone())
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Drop nodes past TTL.
    pub fn reap(&mut self) -> usize {
        let before = self.nodes.len();
        self.nodes.retain(|id, n| {
            let ok = n.is_fresh(self.ttl);
            if !ok {
                warn!(%id, "reaping stale worker");
            }
            ok
        });
        before - self.nodes.len()
    }
}

/// Static peer list (file or env).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticPeerList {
    pub workers: Vec<WorkerNode>,
}

impl StaticPeerList {
    pub fn from_json_str(s: &str) -> Result<Self> {
        serde_json::from_str(s)
            .map_err(|e| LuminaError::Config(format!("peer list json: {e}")))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| LuminaError::Config(format!("read peers: {e}")))?;
        Self::from_json_str(&data)
    }

    /// Parse `id=http://host:port,id2=http://host2:port` style lists.
    pub fn from_env_csv(s: &str) -> Result<Self> {
        let mut workers = Vec::new();
        for part in s.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (id, endpoint) = part.split_once('=').ok_or_else(|| {
                LuminaError::Config(format!(
                    "peer entry must be id=endpoint, got '{part}'"
                ))
            })?;
            workers.push(WorkerNode::new(id.trim(), endpoint.trim()));
        }
        Ok(Self { workers })
    }

    pub fn into_registry(self, ttl: Duration) -> WorkerRegistry {
        let mut reg = WorkerRegistry::new().with_ttl(ttl);
        for w in self.workers {
            reg.register(w);
        }
        reg
    }
}

/// Load discovery config from env:
/// - `LUMINA_WORKERS` — CSV `id=http://host:port,...`
/// - `LUMINA_WORKERS_FILE` — path to JSON peer list
/// - `LUMINA_WORKER_TTL_SECS` — optional TTL (default 30)
pub fn registry_from_env() -> Result<WorkerRegistry> {
    let ttl_secs: u64 = std::env::var("LUMINA_WORKER_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let ttl = Duration::from_secs(ttl_secs);

    if let Ok(path) = std::env::var("LUMINA_WORKERS_FILE") {
        let list = StaticPeerList::from_file(path)?;
        return Ok(list.into_registry(ttl));
    }
    if let Ok(csv) = std::env::var("LUMINA_WORKERS") {
        let list = StaticPeerList::from_env_csv(&csv)?;
        return Ok(list.into_registry(ttl));
    }
    Ok(WorkerRegistry::new().with_ttl(ttl))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_pick() {
        let mut reg = WorkerRegistry::new();
        reg.register(
            WorkerNode::new("a", "http://127.0.0.1:50051").with_models(vec!["m1".into()]),
        );
        reg.register(
            WorkerNode::new("b", "http://127.0.0.1:50052").with_models(vec!["m1".into()]),
        );
        let p1 = reg.pick("m1").unwrap();
        let p2 = reg.pick("m1").unwrap();
        assert_ne!(p1.id, p2.id); // round-robin
        assert!(reg.pick("unknown").is_none());
    }

    #[test]
    fn csv_peers() {
        let list =
            StaticPeerList::from_env_csv("w1=http://10.0.0.1:50051,w2=http://10.0.0.2:50051")
                .unwrap();
        assert_eq!(list.workers.len(), 2);
        assert_eq!(list.workers[0].id, "w1");
    }

    #[test]
    fn stale_reaped() {
        let mut reg = WorkerRegistry::new().with_ttl(Duration::from_secs(1));
        let mut n = WorkerNode::new("old", "http://x");
        n.last_seen = 1; // ancient
        reg.nodes.insert(n.id.clone(), n);
        assert_eq!(reg.reap(), 1);
        assert!(reg.is_empty());
    }
}
