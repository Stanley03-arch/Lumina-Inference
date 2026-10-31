# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.18)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus + admin HTTP | ✅ |
| gRPC server / client / reflection | ✅ |
| Discovery + scheduler + circuit | ✅ |
| Config + rate limit + retry | ✅ |
| Deadlines + affinity + drain | ✅ |
| Priority + health + warmup | ✅ |
| **Latency histogram (p50/p95/p99)** | ✅ |
| **Request audit log** | ✅ |
| **Model path LRU cache** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```

Docs: [CONFIG](docs/CONFIG.md) · [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
