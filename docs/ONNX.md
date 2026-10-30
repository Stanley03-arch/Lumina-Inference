# ONNX Runtime worker

Lumina can run models through [ONNX Runtime](https://onnxruntime.ai/) via the optional `onnx` Cargo feature.

## Enable

```bash
cargo build --features onnx
cargo test --features onnx
cargo run --features onnx --bin lumina
```

Install the ONNX Runtime shared library for your OS (or let the `ort` crate fetch a compatible build, depending on version).

## Point at a model

```bash
export LUMINA_ONNX_MODEL=/path/to/model.onnx
cargo run --features onnx --bin lumina
```

Without `LUMINA_ONNX_MODEL`, the CLI uses the **echo** `LocalWorker`.

## API sketch

```rust
use lumina_inference::{EngineConfig, InferenceEngine, Metrics, OnnxWorker};
use std::sync::Arc;

let w = OnnxWorker::load("onnx-0", "my-model", "models/model.onnx")?;
let eng = InferenceEngine::with_workers(
    EngineConfig::default(),
    vec![Box::new(w)],
    Arc::new(Metrics::default()),
)?;
```

## Input format (v0.1.2)

Request `input` bytes are interpreted as a **flat little-endian `f32` vector** (length multiple of 4). Output tensors are serialized the same way when possible.

This is intentionally minimal — map named I/O tensors to your model’s graph in a follow-up.

## Without the feature

`OnnxWorker::load` still works if the file exists, but `run_batch` returns:

```text
ONNX support not compiled in; rebuild with `--features onnx`
```
