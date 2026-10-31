# Changelog

## [0.1.14] - 2026-10-31

### Added
- **Circuit breaker** per worker (`CircuitBreaker`, open / half-open / closed)
- Scheduler skips open circuits; `record_success` / `record_failure`
- **Admin HTTP** `GET /workers` (JSON) + `/health` via `spawn_admin_server`

### Changed
- Package version **0.1.14**

## [0.1.13] - 2026-10-31

- Load-aware scheduler

## [0.1.12] — [0.1.0]

- mDNS, gossip, gRPC, ONNX, scaffold
