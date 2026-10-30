//! Lumina-Inference — high-performance distributed inference engine.
//!
//! Core crates expose engine configuration, request batching, and a
//! pluggable model worker interface. Prometheus `/metrics` is built-in.
//! ONNX Runtime: `--features onnx`. gRPC: `--features grpc` (needs `protoc`).

pub mod async_engine;
pub mod batch;
pub mod config;
pub mod engine;
pub mod error;
pub mod metrics;
pub mod onnx_worker;
pub mod prometheus_server;
pub mod registry;
pub mod worker;

#[cfg(feature = "grpc")]
pub mod grpc_server;

pub use async_engine::AsyncInferenceEngine;
pub use config::EngineConfig;
pub use engine::InferenceEngine;
pub use error::{LuminaError, Result};
pub use metrics::{Metrics, MetricsSnapshot};
pub use onnx_worker::OnnxWorker;
pub use prometheus_server::{serve_metrics, spawn_metrics_server};
pub use registry::{ModelInfo, ModelRegistry};
pub use worker::{InferenceWorker, LocalWorker};

#[cfg(feature = "grpc")]
pub use grpc_server::{serve_grpc, serve_grpc_shared, InferenceService};
