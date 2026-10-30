# gRPC serve surface + client

Lumina exposes a gRPC **Inference** service (package `lumina.v1`) when built with the `grpc` feature.

**Server reflection is enabled by default** — `grpcurl` can list services without local `.proto` files.

A typed Rust **client** is available as `lumina_inference::InferenceClient`.

## Requirements

- [Protocol Buffers compiler](https://grpc.io/docs/protoc-installation/) (`protoc`) on `PATH` at **build** time
- Rust toolchain

```bash
sudo apt install protobuf-compiler   # Debian/Ubuntu
cargo build --features grpc
```

## Run the server

```bash
export LUMINA_GRPC_ADDR=127.0.0.1:50051
cargo run --features grpc --bin lumina
```

Optional:

```bash
export LUMINA_METRICS_ADDR=127.0.0.1:9090
export LUMINA_ONNX_MODEL=/path/to/model.onnx   # --features onnx,grpc
```

## Run the client (CLI)

With a server already listening:

```bash
LUMINA_GRPC_CLIENT=http://127.0.0.1:50051 cargo run --features grpc --bin lumina
```

This calls `Health`, `Predict`, and `PredictBatch` and prints results.

## Rust client API

```rust
use lumina_inference::{batch::InferenceRequest, InferenceClient};

#[tokio::main]
async fn main() -> lumina_inference::Result<()> {
    let mut client = InferenceClient::connect("http://127.0.0.1:50051").await?;

    let (ok, version) = client.health().await?;
    println!("ok={ok} version={version}");

    let resp = client
        .predict(InferenceRequest {
            id: "1".into(),
            model: "default".into(),
            input: 1.0f32.to_le_bytes().to_vec(),
        })
        .await?;
    println!("latency_ms={}", resp.latency_ms);

    let batch = client
        .predict_batch(vec![/* ... */])
        .await?;
    Ok(())
}
```

| Method | Description |
|--------|-------------|
| `connect(endpoint)` | Dial `http://host:port` |
| `health()` | `(ok, version)` |
| `predict(req)` | Single `InferenceResponse` |
| `predict_batch(reqs)` | `Vec<InferenceResponse>` |

## Service API

Proto: [`proto/lumina/v1/inference.proto`](../proto/lumina/v1/inference.proto)

| RPC | Description |
|-----|-------------|
| `Predict` | Single request; flushes immediately |
| `PredictBatch` | Multiple requests; flushes remainder |
| `Health` | `{ ok, version }` |

## Reflection / grpcurl

```bash
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext -d '{}' 127.0.0.1:50051 lumina.v1.Inference/Health
grpcurl -plaintext \
  -d '{"id":"1","model":"default","input":"AAAAAA=="}' \
  127.0.0.1:50051 lumina.v1.Inference/Predict
```
