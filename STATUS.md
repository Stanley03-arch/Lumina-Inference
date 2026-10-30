# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.11)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server / client / reflection | ✅ |
| Static peer discovery | ✅ |
| **UDP gossip dynamic discovery** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
LUMINA_GOSSIP=1 cargo test
```

Docs: [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md)

## Next milestones

1. Load-aware scheduling
2. mDNS / DNS-SD (optional)
