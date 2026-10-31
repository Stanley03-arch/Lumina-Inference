# Configuration, rate limits & retries

## Engine config

Load JSON via file or env:

```bash
export LUMINA_CONFIG=/etc/lumina/config.json
# or overrides:
export LUMINA_MAX_BATCH=64
export LUMINA_BATCH_TIMEOUT_MS=10
export LUMINA_WORKER_SLOTS=4
export LUMINA_DEFAULT_MODEL=default
export LUMINA_RATE_LIMIT=100   # tokens/sec
```

### `config.json` example

```json
{
  "max_batch_size": 32,
  "batch_timeout_ms": 8,
  "worker_slots": 2,
  "default_model": "default",
  "max_in_flight": 0,
  "rate_limit_per_sec": 100.0,
  "rate_limit_burst": 200.0
}
```

```rust
use lumina_inference::EngineConfig;
let cfg = EngineConfig::from_env()?;
// or EngineConfig::from_file("config.json")?
```

## Rate limiter

Token bucket — admit or reject before enqueue:

```rust
use lumina_inference::{RateLimitConfig, RateLimiter};

let mut lim = RateLimiter::new(RateLimitConfig {
    rate_per_sec: 50.0,
    burst: 100.0,
});
if lim.allow() {
    // accept request
} else {
    // 429 / backpressure
}
```

## Retry

```rust
use lumina_inference::{retry, retry_async, RetryPolicy};

let policy = RetryPolicy {
    max_attempts: 4,
    initial_backoff_ms: 50,
    max_backoff_ms: 1000,
    multiplier: 2.0,
    jitter: 0.1,
};

let result = retry(&policy, |_attempt| do_remote_call())?;

// async
let result = retry_async(&policy, |_a| async { do_remote_call_async().await }).await?;
```
