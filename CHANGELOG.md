# Changelog

## [0.1.15] - 2026-10-31

### Added
- **Retry** with exponential backoff (`RetryPolicy`, `retry`, `retry_async`)
- **Token-bucket rate limiter** (`RateLimiter`, `RateLimitConfig`)
- **Config loading**: `EngineConfig::from_file` / `from_env` / JSON
- Rate-limit fields on `EngineConfig`
- `docs/CONFIG.md`

### Changed
- Package version **0.1.15**

## [0.1.14] - 2026-10-31

- Circuit breaker + admin `/workers`

## [0.1.13] — [0.1.0]

- Scheduler, discovery, gRPC, ONNX, scaffold
