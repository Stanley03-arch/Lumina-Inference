//! gRPC serve surface (requires `--features grpc` and `protoc` at build time).

use crate::batch::InferenceRequest;
use crate::engine::InferenceEngine;
use crate::error::LuminaError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::info;

pub mod pb {
    tonic::include_proto!("lumina.v1");
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
}

#[tonic::async_trait]
impl Inference for InferenceService {
    async fn predict(
        &self,
        request: Request<PredictRequest>,
    ) -> std::result::Result<Response<PredictResponse>, Status> {
        let r = request.into_inner();
        let req = InferenceRequest {
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
        };

        let mut eng = self.engine.lock().await;
        match eng.submit(req) {
            Ok(Some(batch)) => {
                let resp = batch.into_iter().next().unwrap_or_else(|| {
                    crate::batch::InferenceResponse {
                        id: String::new(),
                        model: String::new(),
                        output: Vec::new(),
                        latency_ms: 0,
                    }
                });
                Ok(Response::new(to_proto(resp)))
            }
            Ok(None) => {
                // Partial batch: flush this single item path for low-latency unary RPC
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
            let req = InferenceRequest {
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
            };
            match eng.submit(req).map_err(to_status)? {
                Some(done) => {
                    responses.extend(done.into_iter().map(to_proto));
                }
                None => {}
            }
        }
        // Flush remainder
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

/// Bind and serve the Inference gRPC API.
pub async fn serve_grpc(addr: SocketAddr, engine: InferenceEngine) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let svc = InferenceService::new(engine);
    info!(%addr, "gRPC Inference service listening");
    tonic::transport::Server::builder()
        .add_service(InferenceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}

/// Serve using a shared engine (e.g. alongside metrics).
pub async fn serve_grpc_shared(
    addr: SocketAddr,
    engine: Arc<Mutex<InferenceEngine>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let svc = InferenceService::from_shared(engine);
    info!(%addr, "gRPC Inference service listening");
    tonic::transport::Server::builder()
        .add_service(InferenceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
