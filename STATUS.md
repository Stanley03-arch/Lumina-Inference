# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.26)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| Stopwatch + bloom + duration | ✅ |
| **Ring buffer** | ✅ |
| **OnceFlag** | ✅ |
| **KeyedLimiter** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
