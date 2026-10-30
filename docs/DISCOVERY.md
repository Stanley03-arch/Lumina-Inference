# Multi-node worker discovery

Lumina tracks remote inference peers via **static lists**, **UDP gossip**, and optional **mDNS/DNS-SD**.

## Concepts

| Type | Role |
|------|------|
| `WorkerNode` | Peer id, gRPC endpoint, models, capacity |
| `WorkerRegistry` | Register / reap / round-robin `pick` |
| `StaticPeerList` | JSON / env CSV |
| `spawn_gossip` | UDP announce + listen |
| `advertise` / `browse_into_registry` | mDNS (`--features mdns`) |
| `RemoteWorker` | gRPC peer worker |

## Static peers

```bash
export LUMINA_WORKERS="gpu-a=http://10.0.0.1:50051,gpu-b=http://10.0.0.2:50051"
export LUMINA_WORKERS_FILE=/etc/lumina/workers.json
export LUMINA_WORKER_TTL_SECS=30
```

## UDP gossip

```bash
export LUMINA_GOSSIP=1
export LUMINA_GOSSIP_PORT=19551
```

See `GossipConfig` / `spawn_gossip` in the crate docs.

## mDNS / DNS-SD (`--features mdns`)

Service type: **`_lumina._tcp.local.`**

TXT records: `id`, `endpoint`, `models`, `capacity`

```bash
cargo build --features mdns
```

### Advertise

```rust
use lumina_inference::{advertise, port_from_endpoint, WorkerNode};

let node = WorkerNode::new("gpu-a", "http://192.168.1.10:50051");
let port = port_from_endpoint(&node.endpoint);
let _adv = advertise(&node, port)?; // keep handle alive
```

### Browse

```rust
use lumina_inference::{browse_into_registry, WorkerRegistry};
use std::time::Duration;

let mut reg = WorkerRegistry::new();
let n = browse_into_registry(&mut reg, Duration::from_secs(3))?;
println!("found {n} peers");
```

### Background browser

```rust
use lumina_inference::{spawn_mdns_browser, SharedRegistry};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

let shared: SharedRegistry = Arc::new(Mutex::new(WorkerRegistry::new()));
let _h = spawn_mdns_browser(Arc::clone(&shared), Duration::from_secs(2));
```

**Notes**

- Requires multicast on the LAN (same caveats as AirPlay/Chromecast).
- Cloud VPCs often block mDNS — use static peers or gossip unicast there.
- `MdnsAdvertiser` unregisters on drop/shutdown.

## RemoteWorker

Needs `--features grpc`. Forwards `predict` / `predict_batch` to a peer.

## Roadmap

- Load-aware scheduling (capacity-weighted pick)
