# Auth, trace IDs & feature flags (v0.1.20+)

## Token authentication

```bash
export LUMINA_API_TOKENS="tok_a,tok_b"
```

```rust
use lumina_inference::TokenAuth;

let auth = TokenAuth::from_env();
// or TokenAuth::new(["tok_a", "tok_b"]);
auth.check(Some("Bearer tok_a"))?;
auth.check(None)?; // error if tokens configured
```

Empty token set + `allow_if_empty` (default) → auth disabled.

## Trace IDs

```rust
use lumina_inference::{new_trace_id, resolve_trace_id, HEADER_TRACE};

let id = resolve_trace_id(headers.get(HEADER_TRACE).map(|s| s.as_str()));
// propagates client `x-request-id` or generates `lum-<ms>-<n>`
```

## Feature flags

```rust
use lumina_inference::{FeatureFlags, Flag};

let mut ff = FeatureFlags::new();
ff.set("async_batch_v2", Flag::percent(25));
if ff.is_enabled("async_batch_v2", &user_id) {
    // rollout path
}
```
