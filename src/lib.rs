//! Lumina-Inference — high-performance distributed inference engine.
//!
//! Core crates expose engine configuration, request batching, and a
//! pluggable model worker interface. Prometheus `/metrics` is built-in.
//! ONNX Runtime: `--features onnx`. gRPC: `--features grpc` (needs `protoc`).
//! mDNS/DNS-SD: `--features mdns`.

pub mod admin_server;
pub mod affinity;
pub mod async_engine;
pub mod batch;
pub mod circuit;
pub mod config;
pub mod discovery;
pub mod drain;
pub mod engine;
pub mod error;
pub mod gossip;
pub mod health;
pub mod mdns_discovery;
pub mod metrics;
pub mod onnx_worker;
pub mod priority;
pub mod prometheus_server;
pub mod rate_limit;
pub mod registry;
pub mod remote_worker;
pub mod retry;
pub mod scheduler;
pub mod tensor;
pub mod warmup;
pub mod worker;

#[cfg(feature = "grpc")]
pub mod grpc_client;
#[cfg(feature = "grpc")]
pub mod grpc_server;

pub use admin_server::spawn_admin_server;
pub use affinity::AffinityTable;
pub use async_engine::AsyncInferenceEngine;
pub use circuit::{CircuitBreaker, CircuitConfig, CircuitState};
pub use config::EngineConfig;
pub use discovery::{registry_from_env, StaticPeerList, WorkerNode, WorkerRegistry};
pub use drain::{DrainGate, DrainPermit};
pub use engine::InferenceEngine;
pub use error::{LuminaError, Result};
pub use gossip::{spawn_gossip, GossipConfig, GossipMessage, SharedRegistry, DEFAULT_GOSSIP_PORT};
pub use health::{probe_grpc, probe_node, probe_tcp, ProbeResult};
pub use mdns_discovery::{
    advertise, browse_into_registry, port_from_endpoint, spawn_mdns_browser, MdnsAdvertiser,
    SERVICE_TYPE,
};
pub use metrics::{Metrics, MetricsSnapshot};
pub use onnx_worker::OnnxWorker;
pub use priority::{Priority, PriorityQueue};
pub use prometheus_server::{serve_metrics, spawn_metrics_server};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use registry::{ModelInfo, ModelRegistry};
pub use remote_worker::RemoteWorker;
pub use retry::{retry, retry_async, RetryPolicy};
pub use scheduler::{Lease, LoadTracker, SchedulePolicy, Scheduler};
pub use tensor::{IoBinding, NamedTensors, TensorDType, TensorSpec};
pub use warmup::{warmup_worker, WarmupConfig};
pub use worker::{InferenceWorker, LocalWorker};

#[cfg(feature = "grpc")]
pub use grpc_client::InferenceClient;
#[cfg(feature = "grpc")]
pub use grpc_server::{
    serve_grpc, serve_grpc_ctrl_c, serve_grpc_shared, GrpcServer, GrpcServerConfig,
    InferenceService,
};
