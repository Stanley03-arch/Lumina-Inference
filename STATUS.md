# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.6)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX worker | ✅ (`--features onnx`) |
| Prometheus `/metrics` | ✅ |
| gRPC server + reflection | ✅ (`--features grpc`) |
| **gRPC client** | ✅ |
| CLI server / client / demo | ✅ |

## Verify

```bash
# Terminal A — server
LUMINA_GRPC_ADDR=127.0.0.1:50051 cargo run --features grpc --bin lumina

# Terminal B — client
LUMINA_GRPC_CLIENT=http://127.0.0.1:50051 cargo run --features grpc --bin lumina
```

Docs: [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md) · [ONNX](docs/ONNX.md)

## Next milestones

1. Named ONNX I/O tensor mapping
2. Multi-node worker discovery
