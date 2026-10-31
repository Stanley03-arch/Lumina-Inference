# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.13)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server / client / reflection | ✅ |
| Static + gossip + mDNS discovery | ✅ |
| **Load-aware scheduling** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
cargo build --features grpc
cargo build --features mdns
```

Docs: [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
