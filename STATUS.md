# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.20)

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
| Histogram + audit + model cache | ✅ |
| Integrity + SLO + JSONL | ✅ |
| **Token auth** | ✅ |
| **Trace IDs** | ✅ |
| **Feature flags** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```

Docs: [CONFIG](docs/CONFIG.md) · [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
