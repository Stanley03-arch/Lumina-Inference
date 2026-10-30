# gRPC serve surface

Lumina exposes a gRPC **Inference** service (package `lumina.v1`) when built with the `grpc` feature.

## Requirements

- [Protocol Buffers compiler](https://grpc.io/docs/protoc-installation/) (`protoc`) on `PATH` at **build** time
- Rust toolchain

```bash
# Debian/Ubuntu example
sudo apt install protobuf-compiler

cargo build --features grpc
cargo run --features grpc --bin lumina
```

## Run the server

```bash
export LUMINA_GRPC_ADDR=127.0.0.1:50051
# optional metrics alongside demo (gRPC mode serves until killed)
cargo run --features grpc --bin lumina
```

Optional:

```bash
export LUMINA_METRICS_ADDR=127.0.0.1:9090
export LUMINA_ONNX_MODEL=/path/to/model.onnx   # needs --features onnx,grpc
```

## Service API

Proto: [`proto/lumina/v1/inference.proto`](../proto/lumina/v1/inference.proto)

| RPC | Description |
|-----|-------------|
| `Predict` | Single request; flushes immediately for unary latency |
| `PredictBatch` | Multiple requests; flushes remaining buffer at end |
| `Health` | `{ ok, version }` |

### `PredictRequest`

| Field | Type | Notes |
|-------|------|-------|
| `id` | string | Auto-generated if empty |
| `model` | string | Defaults to `"default"` |
| `input` | bytes | Opaque / f32-LE for ONNX path |

### `PredictResponse`

| Field | Type |
|-------|------|
| `id` | string |
| `model` | string |
| `output` | bytes |
| `latency_ms` | uint64 |

## Library usage

```rust
use lumina_inference::{serve_grpc, EngineConfig, InferenceEngine};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eng = InferenceEngine::new(EngineConfig::default())?;
    let addr: SocketAddr = "127.0.0.1:50051".parse()?;
    serve_grpc(addr, eng).await?;
    Ok(())
}
```

## grpcurl example

```bash
# List services (reflection not enabled by default — use proto files)
grpcurl -plaintext -proto proto/lumina/v1/inference.proto \
  -d '{"id":"1","model":"default","input":"AAAAAA=="}' \
  127.0.0.1:50051 lumina.v1.Inference/Predict

grpcurl -plaintext -proto proto/lumina/v1/inference.proto \
  -d '{}' 127.0.0.1:50051 lumina.v1.Inference/Health
```
