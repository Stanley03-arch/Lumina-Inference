# ONNX Runtime worker

Lumina runs models through [ONNX Runtime](https://onnxruntime.ai/) via `--features onnx`.

## Enable

```bash
cargo build --features onnx
export LUMINA_ONNX_MODEL=/path/to/model.onnx
cargo run --features onnx --bin lumina
```

## Dtypes (v0.1.10+)

Wire format is **little-endian** raw bytes.

| DType | Size | Helpers |
|-------|------|---------|
| `F32` | 4 | `TensorSpec::f32` |
| `F64` | 8 | `TensorSpec::f64` |
| `I64` | 8 | `TensorSpec::i64` |
| `I32` | 4 | `TensorSpec::i32` |
| `U8` | 1 | `TensorSpec::u8` |

```rust
use lumina_inference::tensor::{IoBinding, TensorDType, TensorSpec};

// Token ids as i64
let io = IoBinding {
    inputs: vec![TensorSpec::i64("input_ids", vec![1, -1])],
    outputs: vec![TensorSpec::f32("logits", vec![1, -1, 32000])],
};

// Or mixed
let io = IoBinding::single("x", "y", vec![1, 128], TensorDType::F64);
```

## Named I/O

```rust
use lumina_inference::{batch::InferenceRequest, tensor::NamedTensors, OnnxWorker};

let worker = OnnxWorker::load_with_io("w0", "model", "model.onnx", io)?;

let mut req = InferenceRequest::new("1", "model", vec![]);
req.named_inputs.insert("input_ids", /* i64 LE bytes */ vec![]);
```

### Defaults

`OnnxWorker::load(...)` uses input `input` / output `output`, shape `[1, -1]`, dtype **f32**.

If `named_inputs` is empty, `request.input` feeds the **first** declared input.

### Dynamic shapes

At most **one** `-1`/`0` dim per tensor; resolved from element count.

## Without the feature

`run_batch` errors until you rebuild with `--features onnx`.
