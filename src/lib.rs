//! Lumina-Inference — high-performance distributed inference engine.
//!
//! Core crates expose engine configuration, request batching, and a
//! pluggable model worker interface. gRPC / ONNX bindings will land in
//! later milestones.

pub mod batch;
pub mod config;
pub mod engine;
pub mod error;
pub mod worker;

pub use config::EngineConfig;
pub use engine::InferenceEngine;
pub use error::{LuminaError, Result};
