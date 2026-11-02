# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.25)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops suite (auth, cache, SLO, …) | ✅ |
| EMA + debounce + bytes_fmt | ✅ |
| **Stopwatch / ScopeTimer** | ✅ |
| **Bloom filter dedup** | ✅ |
| **Duration parse (env)** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
