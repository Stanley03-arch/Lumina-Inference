# Changelog

## [0.1.7] - 2026-10-30

### Added
- **`GrpcServer` / `GrpcServerConfig`** — concurrency limit, TCP keepalive
- **`serve_grpc_ctrl_c`** — graceful shutdown on Ctrl+C
- `serve_with_shutdown` for custom stop signals

### Changed
- Package version **0.1.7**
- CLI server mode uses graceful shutdown

## [0.1.6] - 2026-10-30

- `InferenceClient` + `LUMINA_GRPC_CLIENT`

## [0.1.5] - 2026-10-30

- gRPC server reflection

## [0.1.4] - 2026-10-30

- gRPC Inference service (server)

## [0.1.3] — [0.1.0]

- Metrics, ONNX, engine scaffold
