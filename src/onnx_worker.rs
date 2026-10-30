//! ONNX Runtime-backed worker.
//!
//! Enable with:
//! ```text
//! cargo build --features onnx
//! ```
//! Requires the ONNX Runtime shared library available on the system
//! (see https://onnxruntime.ai/).

use crate::batch::{InferenceRequest, InferenceResponse};
use crate::error::{LuminaError, Result};
use crate::worker::InferenceWorker;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info};

/// Worker that runs an ONNX model via the `ort` crate when the `onnx` feature is on.
pub struct OnnxWorker {
    name: String,
    model_id: String,
    model_path: PathBuf,
    #[cfg(feature = "onnx")]
    session: ort::session::Session,
}

impl OnnxWorker {
    /// Load a model from disk. With `onnx` feature, builds an ORT session;
    /// without it, still constructs the worker but `run_batch` returns a clear error.
    pub fn load(name: impl Into<String>, model_id: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
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

        #[cfg(feature = "onnx")]
        {
            info!(%name, path = %model_path.display(), "loading ONNX session");
            let session = ort::session::Session::builder()
                .map_err(|e| LuminaError::Inference(format!("ort session builder: {e}")))?
                .commit_from_file(&model_path)
                .map_err(|e| LuminaError::Inference(format!("ort load {}: {e}", model_path.display())))?;
            Ok(Self {
                name,
                model_id,
                model_path,
                session,
            })
        }

        #[cfg(not(feature = "onnx"))]
        {
            info!(
                %name,
                path = %model_path.display(),
                "OnnxWorker registered without `onnx` feature (run will error until enabled)"
            );
            Ok(Self {
                name,
                model_id,
                model_path,
            })
        }
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }
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
            // Minimal path: treat each request's input bytes as a flat f32 little-endian
            // vector when length is a multiple of 4; otherwise error.
            // Real deployments should map named I/O tensors via model metadata.
            let mut out = Vec::with_capacity(batch.len());
            for req in batch {
                if req.input.len() % 4 != 0 || req.input.is_empty() {
                    return Err(LuminaError::Inference(format!(
                        "request {}: input must be non-empty multiple of 4 bytes (f32 LE)",
                        req.id
                    )));
                }
                let floats: Vec<f32> = req
                    .input
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();

                debug!(id = %req.id, n = floats.len(), "onnx forward");

                // Single input tensor — shape [1, N] for demo flexibility.
                let input = ort::value::Tensor::from_array(([1usize, floats.len()], floats))
                    .map_err(|e| LuminaError::Inference(format!("tensor: {e}")))?;

                let outputs = self
                    .session
                    .run(ort::inputs![input])
                    .map_err(|e| LuminaError::Inference(format!("ort run: {e}")))?;

                // Serialize first output tensor back to LE f32 bytes when possible.
                let output_bytes = if let Some((_name, val)) = outputs.iter().next() {
                    if let Ok(tensor) = val.try_extract_tensor::<f32>() {
                        let (_shape, data) = tensor;
                        data.iter().flat_map(|f| f.to_le_bytes()).collect()
                    } else {
                        // Fallback: opaque marker
                        b"onnx-output".to_vec()
                    }
                } else {
                    Vec::new()
                };

                out.push(InferenceResponse {
                    id: req.id.clone(),
                    model: req.model.clone(),
                    output: output_bytes,
                    latency_ms: start.elapsed().as_millis() as u64,
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
    fn without_onnx_feature_run_errors() {
        // Create empty temp file so path exists
        let dir = std::env::temp_dir();
        let path = dir.join("lumina_empty.onnx");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"not-a-real-onnx").unwrap();
        drop(f);

        let w = OnnxWorker::load("w0", "demo", &path).unwrap();
        let req = InferenceRequest {
            id: "1".into(),
            model: "demo".into(),
            input: 1.0f32.to_le_bytes().to_vec(),
        };
        let res = w.run_batch(&[req]);
        #[cfg(not(feature = "onnx"))]
        assert!(res.is_err());
        #[cfg(feature = "onnx")]
        {
            // Invalid ONNX file should fail at load or run — load may already fail
            let _ = res;
        }
        let _ = std::fs::remove_file(&path);
    }
}
