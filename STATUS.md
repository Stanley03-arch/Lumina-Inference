# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.16)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus + admin HTTP | ✅ |
| gRPC server / client / reflection | ✅ |
| Discovery (static / gossip / mDNS) | ✅ |
| Scheduler + circuit breaker | ✅ |
| Config + rate limit + retry | ✅ |
| **Request deadlines** | ✅ |
| **Sticky affinity** | ✅ |
| **Graceful drain** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```

Docs: [CONFIG](docs/CONFIG.md) · [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
