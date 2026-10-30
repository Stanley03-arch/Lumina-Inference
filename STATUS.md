# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.10)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors | ✅ |
| **ONNX dtypes f32/f64/i64/i32/u8** | ✅ |
| Prometheus `/metrics` | ✅ |
| gRPC server / client / reflection | ✅ |
| Multi-node discovery + RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo run --bin lumina
```

Docs: [ONNX](docs/ONNX.md) · [DISCOVERY](docs/DISCOVERY.md) · [GRPC](docs/GRPC.md) · [METRICS](docs/METRICS.md)

## Next milestones

1. Dynamic discovery (mDNS / gossip)
2. Load-aware scheduling
