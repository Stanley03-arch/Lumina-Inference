# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.21)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + gRPC + mDNS features | ✅ |
| Discovery + scheduler + circuit | ✅ |
| Config + rate limit + retry | ✅ |
| Deadlines + affinity + drain | ✅ |
| Priority + health + warmup | ✅ |
| Histogram + audit + model cache | ✅ |
| Integrity + SLO + JSONL | ✅ |
| Auth + trace + feature flags | ✅ |
| **Backpressure** | ✅ |
| **Shutdown coordinator** | ✅ |
| **Version / feature summary** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
