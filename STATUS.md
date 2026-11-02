# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.27)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| RingBuf + OnceFlag + KeyedLimiter | ✅ |
| **CountDownLatch** | ✅ |
| **env_util helpers** | ✅ |
| **Fanout broadcaster** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
