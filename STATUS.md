# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.23)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Discovery + scheduler + ops suite | ✅ |
| Memory guard + response cache | ✅ |
| **Adaptive batch sizing** | ✅ |
| **Work queue** | ✅ |
| **Host / worker identity** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
