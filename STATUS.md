# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.32)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| Throttle + WRR + progress | ✅ |
| **Cooldown** | ✅ |
| **Jitter helpers** | ✅ |
| **Request coalescer** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
