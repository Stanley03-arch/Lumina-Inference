# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.31)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| LeakyBucket + Readiness + Intern | ✅ |
| **Throttle** | ✅ |
| **Weighted round-robin** | ✅ |
| **Progress tracker** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
