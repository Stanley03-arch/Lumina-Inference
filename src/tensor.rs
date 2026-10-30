//! Named tensor metadata for ONNX (and future) workers.

use serde::{Deserialize, Serialize};

/// How raw request bytes are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TensorDType {
    #[default]
    F32,
    F64,
    I64,
    I32,
    U8,
}

impl TensorDType {
    pub fn element_size(self) -> usize {
        match self {
            TensorDType::F32 | TensorDType::I32 => 4,
            TensorDType::F64 | TensorDType::I64 => 8,
            TensorDType::U8 => 1,
        }
    }
}

/// Description of one model input or output tensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorSpec {
    pub name: String,
    /// Shape; use `-1` or `0` for a dynamic dimension (filled from data length when possible).
    pub shape: Vec<i64>,
    #[serde(default)]
    pub dtype: TensorDType,
}

impl TensorSpec {
    pub fn f32(name: impl Into<String>, shape: Vec<i64>) -> Self {
        Self {
            name: name.into(),
            shape,
            dtype: TensorDType::F32,
        }
    }

    /// Number of elements implied by shape (dynamic dims treated as 1 for capacity hints).
    pub fn element_count_hint(&self) -> usize {
        self.shape
            .iter()
            .map(|&d| if d <= 0 { 1usize } else { d as usize })
            .product()
    }
}

/// Binding of model I/O names used by [`crate::onnx_worker::OnnxWorker`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoBinding {
    pub inputs: Vec<TensorSpec>,
    pub outputs: Vec<TensorSpec>,
}

impl IoBinding {
    /// Single f32 input/output (common demo shape).
    pub fn single_f32(input: &str, output: &str, shape: Vec<i64>) -> Self {
        Self {
            inputs: vec![TensorSpec::f32(input, shape.clone())],
            outputs: vec![TensorSpec::f32(output, shape)],
        }
    }

    pub fn primary_input(&self) -> Option<&TensorSpec> {
        self.inputs.first()
    }

    pub fn primary_output(&self) -> Option<&TensorSpec> {
        self.outputs.first()
    }
}

/// Optional per-request named payloads (name → raw bytes).
/// When empty, workers fall back to `InferenceRequest.input` for the primary input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamedTensors {
    pub tensors: Vec<(String, Vec<u8>)>,
}

impl NamedTensors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, data: Vec<u8>) {
        self.tensors.push((name.into(), data));
    }

    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.tensors
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| d.as_slice())
    }

    pub fn is_empty(&self) -> bool {
        self.tensors.is_empty()
    }
}
