//! Remote worker — forwards inference to a peer via gRPC (`--features grpc`).

use crate::batch::{InferenceRequest, InferenceResponse};
use crate::discovery::WorkerNode;
use crate::error::{LuminaError, Result};
use crate::tensor::NamedTensors;
use crate::worker::InferenceWorker;
use std::sync::Mutex;

/// InferenceWorker that dials a remote Lumina gRPC endpoint.
///
/// The underlying client is created lazily on first use and cached.
#[cfg(feature = "grpc")]
pub struct RemoteWorker {
    node: WorkerNode,
    client: Mutex<Option<crate::grpc_client::InferenceClient>>,
}

#[cfg(feature = "grpc")]
impl RemoteWorker {
    pub fn new(node: WorkerNode) -> Self {
        Self {
            node,
            client: Mutex::new(None),
        }
    }

    pub fn node(&self) -> &WorkerNode {
        &self.node
    }

    fn endpoint(&self) -> &str {
        &self.node.endpoint
    }
}

#[cfg(feature = "grpc")]
impl InferenceWorker for RemoteWorker {
    fn name(&self) -> &str {
        &self.node.id
    }

    fn run_batch(&self, batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>> {
        // Blocking bridge: gRPC client is async; use a lightweight runtime for the call.
        let endpoint = self.endpoint().to_string();
        let batch = batch.to_vec();

        let rt = tokio::runtime::Handle::try_current().ok();
        let result = if let Some(handle) = rt {
            handle.block_on(async_run(&endpoint, batch))
        } else {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| LuminaError::Internal(format!("tokio rt: {e}")))?;
            rt.block_on(async_run(&endpoint, batch))
        };
        result
    }
}

#[cfg(feature = "grpc")]
async fn async_run(endpoint: &str, batch: Vec<InferenceRequest>) -> Result<Vec<InferenceResponse>> {
    use crate::grpc_client::InferenceClient;
    let mut client = InferenceClient::connect(endpoint).await?;
    if batch.len() == 1 {
        let r = client.predict(batch.into_iter().next().unwrap()).await?;
        Ok(vec![r])
    } else {
        client.predict_batch(batch).await
    }
}

/// Placeholder when gRPC feature is off — always errors on run.
#[cfg(not(feature = "grpc"))]
pub struct RemoteWorker {
    node: WorkerNode,
}

#[cfg(not(feature = "grpc"))]
impl RemoteWorker {
    pub fn new(node: WorkerNode) -> Self {
        Self { node }
    }

    pub fn node(&self) -> &WorkerNode {
        &self.node
    }
}

#[cfg(not(feature = "grpc"))]
impl InferenceWorker for RemoteWorker {
    fn name(&self) -> &str {
        &self.node.id
    }

    fn run_batch(&self, _batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>> {
        let _ = NamedTensors::default();
        Err(LuminaError::Inference(format!(
            "RemoteWorker '{}' needs `--features grpc`",
            self.node.id
        )))
    }
}
