# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.29)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| Sampler + rolling avg + IdGen | ✅ |
| **MinMax stats** | ✅ |
| **DeadlineTimer** | ✅ |
| **Lifecycle Phase** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
