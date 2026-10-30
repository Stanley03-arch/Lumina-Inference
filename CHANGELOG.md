# Changelog

## [0.1.8] - 2026-10-30

### Added
- **Named ONNX I/O**: `TensorSpec`, `IoBinding`, `NamedTensors`
- `OnnxWorker::load_with_io` for multi-input/output models
- `InferenceRequest.named_inputs` / `InferenceResponse.named_outputs`
- Dynamic shape resolution (one dynamic dim per tensor)

### Changed
- Package version **0.1.8**

## [0.1.7] - 2026-10-30

- `GrpcServer` builder + Ctrl+C shutdown

## [0.1.6] - 2026-10-30

- gRPC client

## [0.1.5] — [0.1.0]

- Reflection, server, metrics, ONNX, scaffold
