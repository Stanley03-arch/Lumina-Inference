//! ONNX Runtime-backed worker.
//!
//! Enable with `cargo build --features onnx`.
//! Supports named input/output tensors via [`IoBinding`].

use crate::batch::{InferenceRequest, InferenceResponse};
use crate::error::{LuminaError, Result};
use crate::tensor::{IoBinding, NamedTensors, TensorDType, TensorSpec};
use crate::worker::InferenceWorker;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info};

/// Worker that runs an ONNX model via the `ort` crate when the `onnx` feature is on.
pub struct OnnxWorker {
    name: String,
    model_id: String,
    model_path: PathBuf,
    /// Named I/O mapping; default = single anonymous f32 vector.
    io: IoBinding,
    #[cfg(feature = "onnx")]
    session: ort::session::Session,
}

impl OnnxWorker {
    /// Load with default I/O: primary input name `"input"`, output `"output"`, shape `[1, -1]`.
    pub fn load(
        name: impl Into<String>,
        model_id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::load_with_io(
            name,
            model_id,
            path,
            IoBinding::single_f32("input", "output", vec![1, -1]),
        )
    }

    /// Load with explicit named tensor binding.
    pub fn load_with_io(
        name: impl Into<String>,
        model_id: impl Into<String>,
        path: impl AsRef<Path>,
        io: IoBinding,
    ) -> Result<Self> {
        let name = name.into();
        let model_id = model_id.into();
        let model_path = path.as_ref().to_path_buf();

        if !model_path.exists() {
            return Err(LuminaError::ModelNotFound(format!(
                "{} (path {})",
                model_id,
                model_path.display()
            )));
        }
        if io.inputs.is_empty() {
            return Err(LuminaError::Config(
                "IoBinding must declare at least one input".into(),
            ));
        }

        #[cfg(feature = "onnx")]
        {
            info!(
                %name,
                path = %model_path.display(),
                inputs = ?io.inputs.iter().map(|t| &t.name).collect::<Vec<_>>(),
                outputs = ?io.outputs.iter().map(|t| &t.name).collect::<Vec<_>>(),
                "loading ONNX session"
            );
            let session = ort::session::Session::builder()
                .map_err(|e| LuminaError::Inference(format!("ort session builder: {e}")))?
                .commit_from_file(&model_path)
                .map_err(|e| {
                    LuminaError::Inference(format!("ort load {}: {e}", model_path.display()))
                })?;
            Ok(Self {
                name,
                model_id,
                model_path,
                io,
                session,
            })
        }

        #[cfg(not(feature = "onnx"))]
        {
            info!(
                %name,
                path = %model_path.display(),
                "OnnxWorker registered without `onnx` feature"
            );
            Ok(Self {
                name,
                model_id,
                model_path,
                io,
            })
        }
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn io_binding(&self) -> &IoBinding {
        &self.io
    }
}

fn bytes_for_input(req: &InferenceRequest, spec: &TensorSpec) -> Result<Vec<u8>> {
    if let Some(data) = req.named_inputs.get(&spec.name) {
        return Ok(data.to_vec());
    }
    // Fallback: primary input field for the first declared tensor.
    if !req.input.is_empty() {
        return Ok(req.input.clone());
    }
    Err(LuminaError::Inference(format!(
        "request {}: missing tensor '{}'",
        req.id, spec.name
    )))
}

fn resolve_shape(spec: &TensorSpec, nbytes: usize) -> Result<Vec<usize>> {
    let es = spec.dtype.element_size();
    if nbytes % es != 0 {
        return Err(LuminaError::Inference(format!(
            "tensor '{}': {} bytes not divisible by element size {}",
            spec.name, nbytes, es
        )));
    }
    let nelem = nbytes / es;
    let mut shape: Vec<usize> = spec
        .shape
        .iter()
        .map(|&d| if d <= 0 { 0 } else { d as usize })
        .collect();

    let fixed: usize = shape.iter().filter(|&&d| d > 0).product();
    let dynamic = shape.iter().filter(|&&d| d == 0).count();

    if dynamic == 0 {
        if fixed != nelem && !shape.is_empty() {
            // Allow override: reshape to [1, nelem] when static shape mismatches.
            return Ok(vec![1, nelem]);
        }
        if shape.is_empty() {
            return Ok(vec![nelem]);
        }
        return Ok(shape);
    }
    if dynamic == 1 {
        let dyn_len = if fixed == 0 { nelem } else { nelem / fixed };
        for d in &mut shape {
            if *d == 0 {
                *d = dyn_len;
            }
        }
        return Ok(shape);
    }
    Err(LuminaError::Inference(format!(
        "tensor '{}': only one dynamic dimension supported",
        spec.name
    )))
}

#[cfg(feature = "onnx")]
fn tensor_from_bytes(
    spec: &TensorSpec,
    bytes: &[u8],
) -> Result<ort::value::Value> {
    let shape = resolve_shape(spec, bytes.len())?;
    match spec.dtype {
        TensorDType::F32 => {
            let data: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            ort::value::Tensor::from_array((shape, data))
                .map(|t| t.into())
                .map_err(|e| LuminaError::Inference(format!("tensor {}: {e}", spec.name)))
        }
        TensorDType::U8 => ort::value::Tensor::from_array((shape, bytes.to_vec()))
            .map(|t| t.into())
            .map_err(|e| LuminaError::Inference(format!("tensor {}: {e}", spec.name))),
        other => Err(LuminaError::Inference(format!(
            "dtype {:?} not yet wired for input '{}'",
            other, spec.name
        ))),
    }
}

#[cfg(feature = "onnx")]
fn extract_output_bytes(val: &ort::value::Value) -> Vec<u8> {
    if let Ok((_shape, data)) = val.try_extract_tensor::<f32>() {
        return data.iter().flat_map(|f| f.to_le_bytes()).collect();
    }
    if let Ok((_shape, data)) = val.try_extract_tensor::<u8>() {
        return data.to_vec();
    }
    b"onnx-output".to_vec()
}

impl InferenceWorker for OnnxWorker {
    fn name(&self) -> &str {
        &self.name
    }

    fn run_batch(&self, batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>> {
        let start = Instant::now();

        #[cfg(not(feature = "onnx"))]
        {
            let _ = (batch, start);
            return Err(LuminaError::Inference(
                "ONNX support not compiled in; rebuild with `--features onnx`".into(),
            ));
        }

        #[cfg(feature = "onnx")]
        {
            let mut out = Vec::with_capacity(batch.len());
            for req in batch {
                // Build named inputs from binding + request
                let mut session_inputs: Vec<(String, ort::value::Value)> = Vec::new();
                for (i, spec) in self.io.inputs.iter().enumerate() {
                    let bytes = if i == 0 {
                        bytes_for_input(req, spec)?
                    } else {
                        req.named_inputs
                            .get(&spec.name)
                            .map(|s| s.to_vec())
                            .ok_or_else(|| {
                                LuminaError::Inference(format!(
                                    "request {}: missing named tensor '{}'",
                                    req.id, spec.name
                                ))
                            })?
                    };
                    let value = tensor_from_bytes(spec, &bytes)?;
                    session_inputs.push((spec.name.clone(), value));
                }

                debug!(
                    id = %req.id,
                    inputs = ?session_inputs.iter().map(|(n, _)| n).collect::<Vec<_>>(),
                    "onnx forward"
                );

                let outputs = self
                    .session
                    .run(session_inputs)
                    .map_err(|e| LuminaError::Inference(format!("ort run: {e}")))?;

                let mut named = NamedTensors::new();
                let mut primary = Vec::new();

                if self.io.outputs.is_empty() {
                    // Take first session output
                    if let Some((name, val)) = outputs.iter().next() {
                        let bytes = extract_output_bytes(val);
                        named.insert(name.to_string(), bytes.clone());
                        primary = bytes;
                    }
                } else {
                    for (i, spec) in self.io.outputs.iter().enumerate() {
                        let bytes = outputs
                            .get(spec.name.as_str())
                            .map(extract_output_bytes)
                            .or_else(|| {
                                outputs
                                    .iter()
                                    .nth(i)
                                    .map(|(_, v)| extract_output_bytes(v))
                            })
                            .unwrap_or_default();
                        if i == 0 {
                            primary = bytes.clone();
                        }
                        named.insert(spec.name.clone(), bytes);
                    }
                }

                out.push(InferenceResponse {
                    id: req.id.clone(),
                    model: req.model.clone(),
                    output: primary,
                    latency_ms: start.elapsed().as_millis() as u64,
                    named_outputs: named,
                });
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_file_errors() {
        let err = OnnxWorker::load("w", "m", "/no/such/model.onnx");
        assert!(err.is_err());
    }

    #[test]
    fn resolve_shape_dynamic() {
        let spec = TensorSpec::f32("x", vec![1, -1]);
        let shape = resolve_shape(&spec, 16).unwrap(); // 4 f32s
        assert_eq!(shape, vec![1, 4]);
    }

    #[test]
    fn without_onnx_feature_run_errors() {
        let dir = std::env::temp_dir();
        let path = dir.join("lumina_empty.onnx");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"not-a-real-onnx").unwrap();
        drop(f);

        let w = OnnxWorker::load("w0", "demo", &path).unwrap();
        let req = InferenceRequest::new("1", "demo", 1.0f32.to_le_bytes().to_vec());
        let res = w.run_batch(&[req]);
        #[cfg(not(feature = "onnx"))]
        assert!(res.is_err());
        #[cfg(feature = "onnx")]
        {
            let _ = res;
        }
        let _ = std::fs::remove_file(&path);
    }
}
