# Multi-node worker discovery

Lumina can track remote inference peers and route work to them.

## Concepts

| Type | Role |
|------|------|
| `WorkerNode` | Peer id, gRPC endpoint, models, capacity, last_seen |
| `WorkerRegistry` | Register / heartbeat / reap / round-robin `pick` |
| `StaticPeerList` | Load peers from JSON file or env CSV |
| `RemoteWorker` | `InferenceWorker` that calls a peer over gRPC |

## Environment

```bash
# CSV peers
export LUMINA_WORKERS="gpu-a=http://10.0.0.1:50051,gpu-b=http://10.0.0.2:50051"

# Or JSON file
export LUMINA_WORKERS_FILE=/etc/lumina/workers.json

# Stale after N seconds (default 30)
export LUMINA_WORKER_TTL_SECS=30
```

### `workers.json` example

```json
{
  "workers": [
    {
      "id": "gpu-a",
      "endpoint": "http://10.0.0.1:50051",
      "models": ["default", "resnet"],
      "capacity": 8
    },
    {
      "id": "gpu-b",
      "endpoint": "http://10.0.0.2:50051",
      "models": ["*"],
      "capacity": 4
    }
  ]
}
```

(`"*"` matches any model.)

## Library usage

```rust
use lumina_inference::{
    discovery::{registry_from_env, WorkerNode, WorkerRegistry},
    remote_worker::RemoteWorker,
    worker::InferenceWorker,
    EngineConfig, InferenceEngine, Metrics,
};
use std::sync::Arc;
use std::time::Duration;

let mut reg = registry_from_env()?;
// or: reg.register(WorkerNode::new("local", "http://127.0.0.1:50051"));

let mut workers: Vec<Box<dyn InferenceWorker>> = Vec::new();
for node in reg.healthy() {
    workers.push(Box::new(RemoteWorker::new(node.clone())));
}

let eng = InferenceEngine::with_workers(
    EngineConfig::default(),
    workers,
    Arc::new(Metrics::default()),
)?;
```

### Round-robin pick

```rust
if let Some(node) = reg.pick("resnet") {
    let remote = RemoteWorker::new(node);
    // use as InferenceWorker
}
reg.reap(); // drop stale nodes
```

## Requirements

- `RemoteWorker` needs `--features grpc` (and a running peer with `LUMINA_GRPC_ADDR`).
- Discovery itself has **no** extra features — pure registry + static config.

## Roadmap

- UDP / mDNS announce
- etcd / Consul backends
- Load-aware scheduling (not only RR)
