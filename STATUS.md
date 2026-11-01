# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.22)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Discovery + scheduler + circuit | ✅ |
| Auth + trace + flags | ✅ |
| Backpressure + shutdown + version | ✅ |
| **Memory guard** | ✅ |
| **Response cache** | ✅ |
| **Window counter (QPS)** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
