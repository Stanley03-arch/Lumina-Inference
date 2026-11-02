# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.24)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Discovery + scheduler + ops suite | ✅ |
| Adaptive batch + work queue | ✅ |
| **EMA latency smoother** | ✅ |
| **Debounce gate** | ✅ |
| **Byte size format/parse** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
