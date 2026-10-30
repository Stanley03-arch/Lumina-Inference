# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.1)

| Piece | Status |
|-------|--------|
| `Cargo.toml` | ✅ |
| Config + errors | ✅ |
| Batch accumulator | ✅ |
| Local echo worker | ✅ (stand-in for ONNX) |
| InferenceEngine | ✅ |
| Metrics counters | ✅ |
| Model registry | ✅ |
| Async engine + timeout flush | ✅ |
| CLI `lumina` | ✅ |
| gRPC / ONNX / Prometheus export | 🔜 next milestones |

## Verify

```bash
git clone https://github.com/Stanley03-arch/Lumina-Inference.git
cd Lumina-Inference
cargo test
cargo run --bin lumina
```

## Next milestones

1. ONNX Runtime worker
2. Prometheus `/metrics` HTTP export
3. gRPC serve surface
4. Multi-node worker discovery

See [CHANGELOG.md](CHANGELOG.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
