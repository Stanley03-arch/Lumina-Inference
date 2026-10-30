# ONNX Runtime worker

Lumina runs models through [ONNX Runtime](https://onnxruntime.ai/) via `--features onnx`.

## Enable

```bash
cargo build --features onnx
cargo run --features onnx --bin lumina
```

```bash
export LUMINA_ONNX_MODEL=/path/to/model.onnx
cargo run --features onnx --bin lumina
```

## Named I/O (v0.1.8+)

Bind tensor names and shapes with `IoBinding` / `TensorSpec`:

```rust
use lumina_inference::{
    batch::InferenceRequest,
    tensor::{IoBinding, NamedTensors, TensorSpec},
    OnnxWorker,
};

let io = IoBinding {
    inputs: vec![
        TensorSpec::f32("images", vec![1, 3, 224, 224]),
        TensorSpec::f32("mask", vec![1, 1, 224, 224]),
    ],
    outputs: vec![TensorSpec::f32("logits", vec![1, 1000])],
};

let worker = OnnxWorker::load_with_io("w0", "resnet", "models/resnet.onnx", io)?;

let mut req = InferenceRequest::new("1", "resnet", vec![]);
req.named_inputs.insert("images", /* f32 LE bytes */ vec![]);
req.named_inputs.insert("mask", vec![]);
```

### Defaults

`OnnxWorker::load(...)` uses:

- input name: `input`
- output name: `output`
- shape: `[1, -1]` (dynamic length from byte payload)

If `named_inputs` is empty, `request.input` is used for the **first** declared input.

### Dtypes

| DType | Status |
|-------|--------|
| `F32` | ✅ |
| `U8` | ✅ |
| `F64` / `I32` / `I64` | reserved |

Dynamic dims: at most **one** `-1`/`0` per tensor; resolved from element count.

## Without the feature

`OnnxWorker::load` works if the file exists, but `run_batch` errors until you rebuild with `--features onnx`.
