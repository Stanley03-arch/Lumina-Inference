# Prometheus metrics

Lumina exposes engine counters in [Prometheus text format](https://prometheus.io/docs/instrumenting/exposition_formats/) over HTTP.

## Enable

```bash
export LUMINA_METRICS_ADDR=127.0.0.1:9090
cargo run --bin lumina
```

Then scrape:

```bash
curl -s http://127.0.0.1:9090/metrics
```

Health check: `GET /health` → `ok`.

## Series

| Metric | Type | Description |
|--------|------|-------------|
| `lumina_requests_total` | counter | Requests submitted |
| `lumina_batches_total` | counter | Batches executed |
| `lumina_batch_items_total` | counter | Items across all batches |
| `lumina_errors_total` | counter | Submit/flush errors |
| `lumina_inference_latency_ms_sum` | summary | Sum of batch latencies (ms) |
| `lumina_inference_latency_ms_count` | summary | Number of latency samples |

## Library usage

```rust
use lumina_inference::{Metrics, spawn_metrics_server};
use std::net::SocketAddr;
use std::sync::Arc;

let metrics = Arc::new(Metrics::default());
let addr: SocketAddr = "127.0.0.1:9090".parse()?;
spawn_metrics_server(addr, metrics.clone()).await?;

// share `metrics` with InferenceEngine::with_workers / with_metrics
println!("{}", metrics.encode_prometheus());
```

No extra crates: the server is a small Tokio TCP listener.

## Prometheus scrape config example

```yaml
scrape_configs:
  - job_name: lumina
    static_configs:
      - targets: ["127.0.0.1:9090"]
```
