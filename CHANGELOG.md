# Changelog

## [0.1.3] - 2026-10-30

### Added
- Prometheus text exposition (`Metrics::encode_prometheus`)
- HTTP `/metrics` + `/health` via `spawn_metrics_server`
- Batch latency summary (`lumina_inference_latency_ms_*`)
- CLI: `LUMINA_METRICS_ADDR=host:port` to serve metrics
- `docs/METRICS.md`

### Changed
- Package version **0.1.3**
- Engine records latency on successful flush

## [0.1.2] - 2026-10-30

- `InferenceWorker` trait, `OnnxWorker`, `--features onnx`

## [0.1.1] - 2026-10-30

- Metrics counters, model registry, async engine

## [0.1.0] - 2026-10-30

- Initial Rust engine scaffold
