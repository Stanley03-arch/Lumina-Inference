# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.5)

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
| gRPC Inference API | ✅ (`--features grpc`) |
| **gRPC reflection** | ✅ |
| CLI `lumina` | ✅ |

## Verify

```bash
cargo test
cargo run --bin lumina

# gRPC + reflection (requires protoc)
LUMINA_GRPC_ADDR=127.0.0.1:50051 cargo run --features grpc --bin lumina
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext -d '{}' 127.0.0.1:50051 lumina.v1.Inference/Health
```

Docs: [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md) · [ONNX](docs/ONNX.md)

## Next milestones

1. Named ONNX I/O tensor mapping
2. Multi-node worker discovery
