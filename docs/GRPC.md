# gRPC server + client

Lumina exposes a gRPC **Inference** service (package `lumina.v1`) when built with the `grpc` feature.

**Server reflection is enabled by default.** A typed Rust client is available as `InferenceClient`.

## Requirements

- `protoc` on `PATH` at build time
- `cargo build --features grpc`

## Run the server

```bash
export LUMINA_GRPC_ADDR=127.0.0.1:50051
cargo run --features grpc --bin lumina
# Ctrl+C for graceful shutdown
```

Optional metrics alongside the server:

```bash
export LUMINA_METRICS_ADDR=127.0.0.1:9090
export LUMINA_GRPC_ADDR=127.0.0.1:50051
cargo run --features grpc --bin lumina
```

## Server API (library)

```rust
use lumina_inference::{serve_grpc_ctrl_c, EngineConfig, InferenceEngine, GrpcServer, GrpcServerConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eng = InferenceEngine::new(EngineConfig::default())?;
    let addr: SocketAddr = "127.0.0.1:50051".parse()?;

    // Simple: Ctrl+C shutdown
    serve_grpc_ctrl_c(addr, eng).await?;

    // Or builder:
    // let eng = InferenceEngine::new(EngineConfig::default())?;
    // let mut cfg = GrpcServerConfig::new(addr);
    // cfg.concurrency_limit = Some(128);
    // GrpcServer::new(cfg, eng).serve().await?;
    Ok(())
}
```

| API | Role |
|-----|------|
| `serve_grpc` | Block until killed |
| `serve_grpc_ctrl_c` | Graceful stop on Ctrl+C |
| `serve_grpc_shared` | Shared `Arc<Mutex<Engine>>` |
| `GrpcServer` / `GrpcServerConfig` | Concurrency + keepalive + custom shutdown |

## Run the client

```bash
LUMINA_GRPC_CLIENT=http://127.0.0.1:50051 cargo run --features grpc --bin lumina
```

```rust
use lumina_inference::{batch::InferenceRequest, InferenceClient};

let mut client = InferenceClient::connect("http://127.0.0.1:50051").await?;
let (ok, version) = client.health().await?;
let resp = client.predict(InferenceRequest { /* ... */ }).await?;
```

## RPCs

| RPC | Description |
|-----|-------------|
| `Predict` | Single request |
| `PredictBatch` | Multiple requests |
| `Health` | `{ ok, version }` |

## Reflection / grpcurl

```bash
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext -d '{}' 127.0.0.1:50051 lumina.v1.Inference/Health
```
