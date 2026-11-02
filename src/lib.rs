//! Lumina-Inference — high-performance distributed inference engine.
//!
//! Core crates expose engine configuration, request batching, and a
//! pluggable model worker interface. Prometheus `/metrics` is built-in.
//! ONNX Runtime: `--features onnx`. gRPC: `--features grpc` (needs `protoc`).
//! mDNS/DNS-SD: `--features mdns`.

pub mod adaptive_batch;
pub mod admin_server;
pub mod affinity;
pub mod async_engine;
pub mod audit;
pub mod auth;
pub mod backpressure;
pub mod batch;
pub mod bytes_fmt;
pub mod circuit;
pub mod config;
pub mod debounce;
pub mod discovery;
pub mod drain;
pub mod ema;
pub mod engine;
pub mod error;
pub mod flags;
pub mod gossip;
pub mod health;
pub mod histogram;
pub mod host_info;
pub mod integrity;
pub mod jsonl;
pub mod mdns_discovery;
pub mod memory_guard;
pub mod metrics;
pub mod model_cache;
pub mod onnx_worker;
pub mod priority;
pub mod prometheus_server;
pub mod rate_limit;
pub mod registry;
pub mod remote_worker;
pub mod response_cache;
pub mod retry;
pub mod scheduler;
pub mod shutdown;
pub mod slo;
pub mod tensor;
pub mod trace;
pub mod version_info;
pub mod warmup;
pub mod window_counter;
pub mod work_queue;
pub mod worker;

#[cfg(feature = "grpc")]
pub mod grpc_client;
#[cfg(feature = "grpc")]
pub mod grpc_server;

pub use adaptive_batch::AdaptiveBatch;
pub use admin_server::spawn_admin_server;
pub use affinity::AffinityTable;
pub use async_engine::AsyncInferenceEngine;
pub use audit::{AuditEvent, AuditLog};
pub use auth::TokenAuth;
pub use backpressure::{Backpressure, Permit as BackpressurePermit};
pub use bytes_fmt::{format_bytes, parse_bytes};
pub use circuit::{CircuitBreaker, CircuitConfig, CircuitState};
pub use config::EngineConfig;
pub use debounce::Debounce;
pub use discovery::{registry_from_env, StaticPeerList, WorkerNode, WorkerRegistry};
pub use drain::{DrainGate, DrainPermit};
pub use ema::Ema;
pub use engine::InferenceEngine;
pub use error::{LuminaError, Result};
pub use flags::{FeatureFlags, Flag};
pub use gossip::{spawn_gossip, GossipConfig, GossipMessage, SharedRegistry, DEFAULT_GOSSIP_PORT};
pub use health::{probe_grpc, probe_node, probe_tcp, ProbeResult};
pub use histogram::{Histogram, DEFAULT_BOUNDS_MS};
pub use host_info::{default_endpoint, default_worker_id, hostname, pid_string};
pub use integrity::{sha256_file, verify_model, ModelDigest};
pub use jsonl::JsonlWriter;
pub use mdns_discovery::{
    advertise, browse_into_registry, port_from_endpoint, spawn_mdns_browser, MdnsAdvertiser,
    SERVICE_TYPE,
};
pub use memory_guard::MemoryGuard;
pub use metrics::{Metrics, MetricsSnapshot};
pub use model_cache::ModelCache;
pub use onnx_worker::OnnxWorker;
pub use priority::{Priority, PriorityQueue};
pub use prometheus_server::{serve_metrics, spawn_metrics_server};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use registry::{ModelInfo, ModelRegistry};
pub use remote_worker::RemoteWorker;
pub use response_cache::ResponseCache;
pub use retry::{retry, retry_async, RetryPolicy};
pub use scheduler::{Lease, LoadTracker, SchedulePolicy, Scheduler};
pub use shutdown::Shutdown;
pub use slo::SloTracker;
pub use tensor::{IoBinding, NamedTensors, TensorDType, TensorSpec};
pub use trace::{new_trace_id, resolve_trace_id, HEADER_TRACE, HEADER_TRACE_ALT};
pub use version_info::{feature_summary, package_version, version_json};
pub use warmup::{warmup_worker, WarmupConfig};
pub use window_counter::WindowCounter;
pub use work_queue::WorkQueue;
pub use worker::{InferenceWorker, LocalWorker};

#[cfg(feature = "grpc")]
pub use grpc_client::InferenceClient;
#[cfg(feature = "grpc")]
pub use grpc_server::{
    serve_grpc, serve_grpc_ctrl_c, serve_grpc_shared, GrpcServer, GrpcServerConfig,
    InferenceService,
};
