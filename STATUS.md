# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.14)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server / client / reflection | ✅ |
| Static + gossip + mDNS discovery | ✅ |
| Load-aware scheduling | ✅ |
| **Circuit breaker** | ✅ |
| **Admin `/workers`** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```

Docs: [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
