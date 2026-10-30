# Multi-node worker discovery

Lumina tracks remote inference peers via **static lists** and optional **UDP gossip**.

## Concepts

| Type | Role |
|------|------|
| `WorkerNode` | Peer id, gRPC endpoint, models, capacity, last_seen |
| `WorkerRegistry` | Register / heartbeat / reap / round-robin `pick` |
| `StaticPeerList` | JSON file or env CSV |
| `GossipConfig` / `spawn_gossip` | UDP announce + listen (dynamic) |
| `RemoteWorker` | `InferenceWorker` that calls a peer over gRPC |

## Static peers

```bash
export LUMINA_WORKERS="gpu-a=http://10.0.0.1:50051,gpu-b=http://10.0.0.2:50051"
# or
export LUMINA_WORKERS_FILE=/etc/lumina/workers.json
export LUMINA_WORKER_TTL_SECS=30
```

```json
{
  "workers": [
    {
      "id": "gpu-a",
      "endpoint": "http://10.0.0.1:50051",
      "models": ["default", "resnet"],
      "capacity": 8
    }
  ]
}
```

## Dynamic discovery (UDP gossip)

Nodes periodically broadcast a JSON `GossipMessage` and merge peers into the registry.

```bash
export LUMINA_GOSSIP=1
export LUMINA_GOSSIP_PORT=19551          # default
# optional overrides:
# export LUMINA_GOSSIP_BIND=0.0.0.0:19551
# export LUMINA_GOSSIP_ADDR=255.255.255.255:19551
# export LUMINA_GOSSIP_INTERVAL_SECS=5
```

### Library

```rust
use lumina_inference::{
    discovery::{registry_from_env, WorkerNode},
    gossip::{spawn_gossip, GossipConfig},
};

#[tokio::main]
async fn main() -> lumina_inference::Result<()> {
    let local = WorkerNode::new("gpu-a", "http://10.0.0.1:50051")
        .with_models(vec!["default".into()]);
    let reg = registry_from_env()?;
    let cfg = GossipConfig::from_env()?.unwrap_or_else(|| GossipConfig::lan(19551));
    let (shared, _handles) = spawn_gossip(cfg, local, reg).await?;

    // Later: list healthy peers
    let peers = shared.lock().await.healthy();
    for p in peers {
        println!("{} -> {}", p.id, p.endpoint);
    }
    Ok(())
}
```

| Env | Meaning |
|-----|---------|
| `LUMINA_GOSSIP=1` | Enable gossip helpers from env |
| `LUMINA_GOSSIP_PORT` | UDP port (default `19551`) |
| `LUMINA_GOSSIP_BIND` | Listen bind address |
| `LUMINA_GOSSIP_ADDR` | Announce destination (broadcast/unicast) |
| `LUMINA_GOSSIP_INTERVAL_SECS` | Announce period |

**Notes**

- Uses UDP **broadcast** by default (`255.255.255.255`) — best on a LAN; cloud VPCs often block broadcast (use unicast `LUMINA_GOSSIP_ADDR` or static peers).
- Self-announces are ignored via `self_id`.
- Stale nodes are reaped on each received packet using registry TTL.

## RemoteWorker

Needs `--features grpc`. Forwards `predict` / `predict_batch` to a peer endpoint.

## Roadmap

- mDNS / DNS-SD
- Load-aware scheduling (capacity-weighted pick)
