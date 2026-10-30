# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.2)

| Piece | Status |
|-------|--------|
| Config + errors + metrics | ✅ |
| Batch accumulator | ✅ |
| `InferenceWorker` trait | ✅ |
| Local echo worker | ✅ |
| **ONNX Runtime worker** | ✅ (`--features onnx`) |
| Model registry | ✅ |
| Async engine + timeout flush | ✅ |
| CLI `lumina` | ✅ |
| Prometheus `/metrics` | 🔜 |
| gRPC serve | 🔜 |

## Verify

```bash
cargo test
cargo run --bin lumina

# With ONNX Runtime
cargo run --features onnx --bin lumina
LUMINA_ONNX_MODEL=/path/to/model.onnx cargo run --features onnx --bin lumina
```

See [docs/ONNX.md](docs/ONNX.md).

## Next milestones

1. Prometheus `/metrics` HTTP export
2. gRPC serve surface
3. Named ONNX I/O tensor mapping
4. Multi-node worker discovery
