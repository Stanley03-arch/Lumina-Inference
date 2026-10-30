# Changelog

## [0.1.0] - 2026-10-30

### Added — Rust engine scaffold
- `Cargo.toml` package `lumina-inference`
- Modules: `config`, `error`, `batch`, `worker`, `engine`
- `LocalWorker` echo backend (placeholder for ONNX)
- `InferenceEngine` with batching + round-robin workers
- CLI binary `lumina` demo run
- Unit tests for batch fill and engine flush

### Docs
- `.gitignore`, `STATUS.md`, CONTRIBUTING updates

## Vision (from README)

- Ultra-low latency distributed inference in Rust
- Efficient batching, model versioning, Prometheus metrics
- gRPC interface + ONNX Runtime integration
