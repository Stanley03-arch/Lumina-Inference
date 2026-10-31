# Status — Lumina-Inference

**High-performance distributed inference engine (Rust).**

## Current state (v0.1.15)

| Piece | Status |
|-------|--------|
| Engine + batching + metrics | ✅ |
| ONNX + named tensors + dtypes | ✅ |
| Prometheus + admin HTTP | ✅ |
| gRPC server / client / reflection | ✅ |
| Discovery (static / gossip / mDNS) | ✅ |
| Load-aware scheduler + circuit breaker | ✅ |
| **Config file + env** | ✅ |
| **Rate limiter** | ✅ |
| **Retry / backoff** | ✅ |
| RemoteWorker | ✅ |
| CLI | ✅ |

## Verify

```bash
cargo test
cargo build
```

Docs: [CONFIG](docs/CONFIG.md) · [DISCOVERY](docs/DISCOVERY.md) · [ONNX](docs/ONNX.md) · [GRPC](docs/GRPC.md)
