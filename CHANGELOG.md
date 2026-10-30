# Changelog

## [0.1.2] - 2026-10-30

### Added
- `InferenceWorker` trait for pluggable backends
- `OnnxWorker` with optional `ort` integration (`--features onnx`)
- `InferenceEngine::with_workers` for custom pools
- `docs/ONNX.md` setup guide
- CLI: `LUMINA_ONNX_MODEL` env to load an ONNX file

### Changed
- Default engine still uses `LocalWorker` echo backend
- Package version **0.1.2**

## [0.1.1] - 2026-10-30

### Added
- In-process `Metrics` counters
- `ModelRegistry`
- `AsyncInferenceEngine` with batch-timeout flush

## [0.1.0] - 2026-10-30

### Added
- Initial Rust engine scaffold (config, batch, worker, engine, CLI)
