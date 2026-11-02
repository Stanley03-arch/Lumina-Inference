# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.28)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| Latch + env_util + fanout | ✅ |
| **Sampling gates** | ✅ |
| **Rolling average** | ✅ |
| **IdGen** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
