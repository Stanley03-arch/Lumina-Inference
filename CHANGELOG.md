# Changelog

## [0.1.9] - 2026-10-30

### Added
- **Multi-node worker discovery**: `WorkerNode`, `WorkerRegistry`, `StaticPeerList`
- Env loaders: `LUMINA_WORKERS`, `LUMINA_WORKERS_FILE`, `LUMINA_WORKER_TTL_SECS`
- `RemoteWorker` — gRPC-backed `InferenceWorker` for peers
- Round-robin `pick`, heartbeat, TTL reap
- `docs/DISCOVERY.md`

### Changed
- Package version **0.1.9**

## [0.1.8] - 2026-10-30

- Named ONNX I/O tensors

## [0.1.7] — [0.1.0]

- gRPC, metrics, ONNX, engine scaffold
