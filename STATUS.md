# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.4)

| Piece | Status |
|-------|--------|
| Config + errors + metrics | ✅ |
| Batch accumulator | ✅ |
| `InferenceWorker` trait | ✅ |
| Local echo worker | ✅ |
| ONNX Runtime worker | ✅ (`--features onnx`) |
| Model registry | ✅ |
| Async engine + timeout flush | ✅ |
| Prometheus `/metrics` | ✅ |
| **gRPC Inference API** | ✅ (`--features grpc`) |
| CLI `lumina` | ✅ |

## Verify

```bash
cargo test
cargo run --bin lumina

# Metrics
LUMINA_METRICS_ADDR=127.0.0.1:9090 cargo run --bin lumina

# gRPC (requires protoc)
cargo build --features grpc
LUMINA_GRPC_ADDR=127.0.0.1:50051 cargo run --features grpc --bin lumina
```

Docs: [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md) · [ONNX](docs/ONNX.md)

## Next milestones

1. Named ONNX I/O tensor mapping
2. gRPC reflection / richer client examples
3. Multi-node worker discovery
