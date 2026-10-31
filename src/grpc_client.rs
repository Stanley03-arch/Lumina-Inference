//! gRPC client for the Lumina Inference service (`--features grpc`).

use crate::batch::{InferenceRequest, InferenceResponse};
use crate::error::{LuminaError, Result};
use crate::grpc_server::pb::inference_client::InferenceClient as PbClient;
use crate::grpc_server::pb::{
    HealthRequest, PredictBatchRequest, PredictRequest as PbPredictRequest,
};
use crate::tensor::NamedTensors;
use tonic::transport::Channel;

/// High-level client around the generated tonic stubs.
#[derive(Clone)]
pub struct InferenceClient {
    inner: PbClient<Channel>,
}

impl InferenceClient {
    /// Connect to a server endpoint, e.g. `"http://127.0.0.1:50051"`.
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        let inner = PbClient::connect(endpoint.clone())
            .await
            .map_err(|e| LuminaError::Internal(format!("grpc connect {endpoint}: {e}")))?;
        Ok(Self { inner })
    }

    /// Build from an existing channel (advanced).
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            inner: PbClient::new(channel),
        }
    }

    /// Unary predict.
    pub async fn predict(&mut self, req: InferenceRequest) -> Result<InferenceResponse> {
        let pb = PbPredictRequest {
            id: req.id,
            model: req.model,
            input: req.input,
        };
        let resp = self
            .inner
            .predict(pb)
            .await
            .map_err(|e| LuminaError::Inference(format!("Predict: {e}")))?
            .into_inner();
        Ok(InferenceResponse {
            id: resp.id,
            model: resp.model,
            output: resp.output,
            latency_ms: resp.latency_ms,
            named_outputs: NamedTensors::default(),
        })
    }

    /// Batch predict.
    pub async fn predict_batch(
        &mut self,
        requests: Vec<InferenceRequest>,
    ) -> Result<Vec<InferenceResponse>> {
        let pb_reqs: Vec<_> = requests
            .into_iter()
            .map(|r| PbPredictRequest {
                id: r.id,
                model: r.model,
                input: r.input,
            })
            .collect();
        let resp = self
            .inner
            .predict_batch(PredictBatchRequest { requests: pb_reqs })
            .await
            .map_err(|e| LuminaError::Inference(format!("PredictBatch: {e}")))?
            .into_inner();
        Ok(resp
            .responses
            .into_iter()
            .map(|r| InferenceResponse {
                id: r.id,
                model: r.model,
                output: r.output,
                latency_ms: r.latency_ms,
                named_outputs: NamedTensors::default(),
            })
            .collect())
    }

    /// Health probe — returns `(ok, version)`.
    pub async fn health(&mut self) -> Result<(bool, String)> {
        let resp = self
            .inner
            .health(HealthRequest {})
            .await
            .map_err(|e| LuminaError::Internal(format!("Health: {e}")))?
            .into_inner();
        Ok((resp.ok, resp.version))
    }
}
