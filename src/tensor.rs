//! Named tensor metadata for ONNX (and future) workers.

use serde::{Deserialize, Serialize};

/// How raw request bytes are interpreted (little-endian).
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

    pub fn as_str(self) -> &'static str {
        match self {
            TensorDType::F32 => "f32",
            TensorDType::F64 => "f64",
            TensorDType::I64 => "i64",
            TensorDType::I32 => "i32",
            TensorDType::U8 => "u8",
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
    pub fn new(name: impl Into<String>, shape: Vec<i64>, dtype: TensorDType) -> Self {
        Self {
            name: name.into(),
            shape,
            dtype,
        }
    }

    pub fn f32(name: impl Into<String>, shape: Vec<i64>) -> Self {
        Self::new(name, shape, TensorDType::F32)
    }

    pub fn f64(name: impl Into<String>, shape: Vec<i64>) -> Self {
        Self::new(name, shape, TensorDType::F64)
    }

    pub fn i64(name: impl Into<String>, shape: Vec<i64>) -> Self {
        Self::new(name, shape, TensorDType::I64)
    }

    pub fn i32(name: impl Into<String>, shape: Vec<i64>) -> Self {
        Self::new(name, shape, TensorDType::I32)
    }

    pub fn u8(name: impl Into<String>, shape: Vec<i64>) -> Self {
        Self::new(name, shape, TensorDType::U8)
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

    /// Single tensor pair with an explicit dtype.
    pub fn single(input: &str, output: &str, shape: Vec<i64>, dtype: TensorDType) -> Self {
        Self {
            inputs: vec![TensorSpec::new(input, shape.clone(), dtype)],
            outputs: vec![TensorSpec::new(output, shape, dtype)],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_sizes() {
        assert_eq!(TensorDType::F32.element_size(), 4);
        assert_eq!(TensorDType::F64.element_size(), 8);
        assert_eq!(TensorDType::I64.element_size(), 8);
        assert_eq!(TensorDType::I32.element_size(), 4);
        assert_eq!(TensorDType::U8.element_size(), 1);
    }
}
