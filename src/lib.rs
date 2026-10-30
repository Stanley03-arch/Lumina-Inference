//! Lumina-Inference — high-performance distributed inference engine.
//!
//! Core crates expose engine configuration, request batching, and a
//! pluggable model worker interface. gRPC bindings will land in later
//! milestones. ONNX Runtime is available behind the `onnx` feature.

pub mod async_engine;
pub mod batch;
pub mod config;
pub mod engine;
pub mod error;
pub mod metrics;
pub mod onnx_worker;
pub mod registry;
pub mod worker;

pub use async_engine::AsyncInferenceEngine;
pub use config::EngineConfig;
pub use engine::InferenceEngine;
pub use error::{LuminaError, Result};
pub use metrics::{Metrics, MetricsSnapshot};
pub use onnx_worker::OnnxWorker;
pub use registry::{ModelInfo, ModelRegistry};
pub use worker::{InferenceWorker, LocalWorker};
