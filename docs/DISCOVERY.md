# Multi-node worker discovery & scheduling

Lumina tracks peers via **static lists**, **UDP gossip**, and optional **mDNS**, then routes with a **load-aware scheduler** and **circuit breaker**.

## Discovery

| Mechanism | How |
|-----------|-----|
| Static | `LUMINA_WORKERS` / `LUMINA_WORKERS_FILE` |
| Gossip | `LUMINA_GOSSIP=1` (UDP broadcast) |
| mDNS | `--features mdns`, service `_lumina._tcp.local.` |

```bash
export LUMINA_WORKERS="gpu-a=http://10.0.0.1:50051,gpu-b=http://10.0.0.2:50051"
```

## Scheduling

```rust
use lumina_inference::{
    CircuitBreaker, CircuitConfig, SchedulePolicy, Scheduler, WorkerRegistry,
};

let mut sched = Scheduler::new(SchedulePolicy::LeastLoaded)
    .with_circuit(CircuitBreaker::new(CircuitConfig::default()));

if let Some(node) = sched.pick_and_acquire(&registry, "default") {
    match do_infer(&node) {
        Ok(_) => sched.record_success(&node.id),
        Err(_) => sched.record_failure(&node.id),
    }
    sched.release(&node.id);
}
```

| Policy | Behavior |
|--------|----------|
| `RoundRobin` | Even rotation |
| `LeastLoaded` | Lowest `in_flight / capacity` |
| `WeightedCapacity` | Prefer higher remaining capacity |

### Circuit breaker

- Default: **3** failures → **Open** for **30s** → **HalfOpen** trial → close on success
- Open peers are skipped by `Scheduler::pick`

## Admin HTTP

```rust
use lumina_inference::spawn_admin_server;
use std::sync::Arc;
use tokio::sync::Mutex;

let reg = Arc::new(Mutex::new(registry));
spawn_admin_server("127.0.0.1:9100".parse()?, reg, None).await?;
// GET http://127.0.0.1:9100/workers  → JSON
// GET http://127.0.0.1:9100/health   → ok
```

## RemoteWorker

Needs `--features grpc`. Forwards `predict` / `predict_batch` to a peer endpoint.
