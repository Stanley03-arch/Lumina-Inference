# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.17)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus + admin HTTP | ✅ |
| gRPC server / client / reflection | ✅ |
| Discovery + scheduler + circuit | ✅ |
| Config + rate limit + retry | ✅ |
| Deadlines + affinity + drain | ✅ |
| **Priority queue** | ✅ |
| **Health probes** | ✅ |
| **Model warm-up** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```

Docs: [CONFIG](docs/CONFIG.md) · [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
