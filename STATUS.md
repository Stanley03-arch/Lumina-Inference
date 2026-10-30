# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.3)

| Piece | Status |
|-------|--------|
| Config + errors + metrics | ✅ |
| Batch accumulator | ✅ |
| `InferenceWorker` trait | ✅ |
| Local echo worker | ✅ |
| ONNX Runtime worker | ✅ (`--features onnx`) |
| Model registry | ✅ |
| Async engine + timeout flush | ✅ |
| **Prometheus `/metrics`** | ✅ |
| CLI `lumina` | ✅ |
| gRPC serve | 🔜 |

## Verify

```bash
cargo test
cargo run --bin lumina

# Metrics endpoint
LUMINA_METRICS_ADDR=127.0.0.1:9090 cargo run --bin lumina
curl -s http://127.0.0.1:9090/metrics
```

See [docs/METRICS.md](docs/METRICS.md) and [docs/ONNX.md](docs/ONNX.md).

## Next milestones

1. gRPC serve surface
2. Named ONNX I/O tensor mapping
3. Multi-node worker discovery
