# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.30)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| MinMax + DeadlineTimer + Phase | ✅ |
| **LeakyBucket** | ✅ |
| **Readiness checks** | ✅ |
| **StringInterner** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
