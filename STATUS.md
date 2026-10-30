# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.7)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX worker | ✅ (`--features onnx`) |
| Prometheus `/metrics` | ✅ |
| **gRPC server** (builder, reflection, Ctrl+C shutdown) | ✅ |
| gRPC client | ✅ |
| CLI server / client / demo | ✅ |

## Verify

```bash
# Server (Ctrl+C to stop)
LUMINA_GRPC_ADDR=127.0.0.1:50051 cargo run --features grpc --bin lumina

# Client
LUMINA_GRPC_CLIENT=http://127.0.0.1:50051 cargo run --features grpc --bin lumina
```

Docs: [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md) · [ONNX](docs/ONNX.md)
