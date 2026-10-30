# Changelog

## [0.1.4] - 2026-10-30

### Added
- gRPC **Inference** service (`proto/lumina/v1/inference.proto`)
- `Predict`, `PredictBatch`, `Health` RPCs via tonic
- Feature flag `grpc` (needs `protoc` at build time)
- CLI: `LUMINA_GRPC_ADDR=host:port` long-running server
- `docs/GRPC.md`

### Changed
- Package version **0.1.4**

## [0.1.3] - 2026-10-30

- Prometheus `/metrics` HTTP + latency summary

## [0.1.2] - 2026-10-30

- ONNX worker + `InferenceWorker` trait

## [0.1.1] - 2026-10-30

- Metrics, registry, async engine

## [0.1.0] - 2026-10-30

- Initial Rust engine scaffold
