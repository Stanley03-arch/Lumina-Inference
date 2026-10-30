//! gRPC serve surface (requires `--features grpc` and `protoc` at build time).
//! Includes server reflection so tools like `grpcurl` work without local protos.

use crate::batch::InferenceRequest;
use crate::engine::InferenceEngine;
use crate::error::LuminaError;
use crate::tensor::NamedTensors;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::info;

pub mod pb {
    tonic::include_proto!("lumina.v1");

    /// Encoded FileDescriptorSet for gRPC reflection.
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lumina_descriptor");
}

use pb::inference_server::{Inference, InferenceServer};
use pb::{
    HealthRequest, HealthResponse, PredictBatchRequest, PredictBatchResponse, PredictRequest,
    PredictResponse,
};

pub struct InferenceService {
    engine: Arc<Mutex<InferenceEngine>>,
}

impl InferenceService {
    pub fn new(engine: InferenceEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
        }
    }

    pub fn from_shared(engine: Arc<Mutex<InferenceEngine>>) -> Self {
        Self { engine }
    }

    pub fn engine_handle(&self) -> Arc<Mutex<InferenceEngine>> {
        Arc::clone(&self.engine)
    }
}

fn from_pb(r: PredictRequest) -> InferenceRequest {
    InferenceRequest {
        id: if r.id.is_empty() {
            uuid_like()
        } else {
            r.id
        },
        model: if r.model.is_empty() {
            "default".into()
        } else {
            r.model
        },
        input: r.input,
        named_inputs: NamedTensors::default(),
    }
}

#[tonic::async_trait]
impl Inference for InferenceService {
    async fn predict(
        &self,
        request: Request<PredictRequest>,
    ) -> std::result::Result<Response<PredictResponse>, Status> {
        let req = from_pb(request.into_inner());

        let mut eng = self.engine.lock().await;
        match eng.submit(req) {
            Ok(Some(batch)) => {
                let resp = batch.into_iter().next().unwrap_or_else(|| {
                    crate::batch::InferenceResponse::new(String::new(), String::new(), Vec::new(), 0)
                });
                Ok(Response::new(to_proto(resp)))
            }
            Ok(None) => {
                let batch = eng.flush().map_err(to_status)?;
                let resp = batch.into_iter().next().ok_or_else(|| {
                    Status::internal("flush returned empty after unary submit")
                })?;
                Ok(Response::new(to_proto(resp)))
            }
            Err(e) => Err(to_status(e)),
        }
    }

    async fn predict_batch(
        &self,
        request: Request<PredictBatchRequest>,
    ) -> std::result::Result<Response<PredictBatchResponse>, Status> {
        let batch_req = request.into_inner();
        let mut responses = Vec::with_capacity(batch_req.requests.len());
        let mut eng = self.engine.lock().await;

        for r in batch_req.requests {
            let req = from_pb(r);
            match eng.submit(req).map_err(to_status)? {
                Some(done) => {
                    responses.extend(done.into_iter().map(to_proto));
                }
                None => {}
            }
        }
        let rest = eng.flush().map_err(to_status)?;
        responses.extend(rest.into_iter().map(to_proto));

        Ok(Response::new(PredictBatchResponse { responses }))
    }

    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> std::result::Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            ok: true,
            version: env!("CARGO_PKG_VERSION").into(),
        }))
    }
}

fn to_proto(r: crate::batch::InferenceResponse) -> PredictResponse {
    PredictResponse {
        id: r.id,
        model: r.model,
        output: r.output,
        latency_ms: r.latency_ms,
    }
}

fn to_status(e: LuminaError) -> Status {
    match e {
        LuminaError::Config(m) => Status::invalid_argument(m),
        LuminaError::ModelNotFound(m) => Status::not_found(m),
        LuminaError::BatchOverflow { max } => {
            Status::resource_exhausted(format!("batch overflow max={max}"))
        }
        LuminaError::WorkerUnavailable(m) => Status::unavailable(m),
        LuminaError::Inference(m) => Status::internal(m),
        LuminaError::Internal(m) => Status::internal(m),
    }
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("req-{t}")
}

fn reflection_service(
) -> Result<
    tonic_reflection::server::ServerReflectionServer<impl tonic_reflection::server::ServerReflection>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    Ok(svc)
}

/// Tunables for the gRPC server.
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    pub addr: SocketAddr,
    /// Max concurrent requests (tonic concurrency limit). `None` = unlimited.
    pub concurrency_limit: Option<usize>,
    /// TCP / HTTP2 keepalive interval.
    pub tcp_keepalive: Option<Duration>,
}

impl GrpcServerConfig {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            concurrency_limit: Some(256),
            tcp_keepalive: Some(Duration::from_secs(60)),
        }
    }
}

/// Builder that owns the engine and starts Inference + reflection.
pub struct GrpcServer {
    cfg: GrpcServerConfig,
    service: InferenceService,
}

impl GrpcServer {
    pub fn new(cfg: GrpcServerConfig, engine: InferenceEngine) -> Self {
        Self {
            cfg,
            service: InferenceService::new(engine),
        }
    }

    pub fn from_shared(cfg: GrpcServerConfig, engine: Arc<Mutex<InferenceEngine>>) -> Self {
        Self {
            cfg,
            service: InferenceService::from_shared(engine),
        }
    }

    pub fn service(&self) -> &InferenceService {
        &self.service
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.serve_with_shutdown(std::future::pending::<()>()).await
    }

    pub async fn serve_with_shutdown<F>(
        self,
        signal: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let reflection = reflection_service()?;
        let addr = self.cfg.addr;
        let mut builder = tonic::transport::Server::builder();

        if let Some(n) = self.cfg.concurrency_limit {
            builder = builder.concurrency_limit_per_connection(n);
        }
        if let Some(ka) = self.cfg.tcp_keepalive {
            builder = builder.tcp_keepalive(Some(ka));
        }

        info!(
            %addr,
            concurrency = ?self.cfg.concurrency_limit,
            "gRPC Inference + reflection listening"
        );

        builder
            .add_service(InferenceServer::new(self.service))
            .add_service(reflection)
            .serve_with_shutdown(addr, signal)
            .await?;
        info!(%addr, "gRPC server shut down");
        Ok(())
    }
}

pub async fn serve_grpc(
    addr: SocketAddr,
    engine: InferenceEngine,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    GrpcServer::new(GrpcServerConfig::new(addr), engine)
        .serve()
        .await
}

pub async fn serve_grpc_shared(
    addr: SocketAddr,
    engine: Arc<Mutex<InferenceEngine>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    GrpcServer::from_shared(GrpcServerConfig::new(addr), engine)
        .serve()
        .await
}

pub async fn serve_grpc_ctrl_c(
    addr: SocketAddr,
    engine: InferenceEngine,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = GrpcServer::new(GrpcServerConfig::new(addr), engine);
    server
        .serve_with_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl+C received — shutting down gRPC server");
        })
        .await
}
