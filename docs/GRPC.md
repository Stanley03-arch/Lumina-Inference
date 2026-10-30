# gRPC serve surface

Lumina exposes a gRPC **Inference** service (package `lumina.v1`) when built with the `grpc` feature.

**Server reflection is enabled by default** — `grpcurl` can list services and call RPCs without local `.proto` files.

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

## Reflection

The server registers a `FileDescriptorSet` via `tonic-reflection` so clients can discover methods at runtime.

```bash
# List services (no -proto needed)
grpcurl -plaintext 127.0.0.1:50051 list

# Describe a method
grpcurl -plaintext 127.0.0.1:50051 describe lumina.v1.Inference.Predict

# Call Health
grpcurl -plaintext -d '{}' 127.0.0.1:50051 lumina.v1.Inference/Health

# Call Predict (base64 for bytes field)
grpcurl -plaintext \
  -d '{"id":"1","model":"default","input":"AAAAAA=="}' \
  127.0.0.1:50051 lumina.v1.Inference/Predict
```

## Library usage

```rust
use lumina_inference::{serve_grpc, EngineConfig, InferenceEngine};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eng = InferenceEngine::new(EngineConfig::default())?;
    let addr: SocketAddr = "127.0.0.1:50051".parse()?;
    serve_grpc(addr, eng).await?; // includes reflection
    Ok(())
}
```
