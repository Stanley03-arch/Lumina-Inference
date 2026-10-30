# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.8)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX worker | ✅ |
| **Named ONNX I/O tensors** | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server + client + reflection | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
LUMINA_GRPC_ADDR=127.0.0.1:50051 cargo run --features grpc --bin lumina
```

Docs: [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md)

## Next milestones

1. Multi-node worker discovery
2. Richer dtypes (f64 / i64) for ONNX
