# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.33)

| Piece | Status |
|-------|--------|
| Engine + ONNX + gRPC + mDNS | ✅ |
| Ops / observability suite | ✅ |
| Cooldown + jitter + coalesce | ✅ |
| **ExponentialBackoff** | ✅ |
| **HealthScore** | ✅ |
| **DepthGauge** | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```
