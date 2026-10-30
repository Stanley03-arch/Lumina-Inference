# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.9)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server / client / reflection | ✅ |
| **Multi-node worker discovery** | ✅ |
| RemoteWorker (gRPC peer) | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test

# Two nodes example
LUMINA_GRPC_ADDR=127.0.0.1:50051 cargo run --features grpc --bin lumina
LUMINA_WORKERS="a=http://127.0.0.1:50051" cargo test
```

Docs: [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md)

## Next milestones

1. Richer ONNX dtypes (f64 / i64)
2. Dynamic discovery (mDNS / gossip)
3. Load-aware scheduling
