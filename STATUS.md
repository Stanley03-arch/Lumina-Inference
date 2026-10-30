# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.12)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server / client / reflection | ✅ |
| Static + UDP gossip discovery | ✅ |
| **mDNS / DNS-SD** | ✅ (`--features mdns`) |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build --features mdns
```

Docs: [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)

## Next milestones

1. Load-aware scheduling
