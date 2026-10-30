# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.0)

Working scaffold on `master`:

| Piece | Status |
|-------|--------|
| `Cargo.toml` | ✅ |
| Config + errors | ✅ |
| Batch accumulator | ✅ |
| Local echo worker | ✅ (stand-in for ONNX) |
| InferenceEngine | ✅ |
| CLI `lumina` | ✅ |
| gRPC / ONNX / Prometheus | 🔜 next milestones |

## Verify

```bash
git clone https://github.com/Stanley03-arch/Lumina-Inference.git
cd Lumina-Inference
cargo test
cargo run --bin lumina
```

## Next milestones

1. ONNX Runtime worker
2. Async batch timer (`batch_timeout_ms`)
3. gRPC serve surface
4. Prometheus metrics export

See [CHANGELOG.md](CHANGELOG.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
