//! Remote worker — forwards inference to a peer via gRPC (`--features grpc`).

use crate::batch::{InferenceRequest, InferenceResponse};
use crate::discovery::WorkerNode;
use crate::error::{LuminaError, Result};
use crate::worker::InferenceWorker;

/// InferenceWorker that dials a remote Lumina gRPC endpoint.
#[cfg(feature = "grpc")]
pub struct RemoteWorker {
    node: WorkerNode,
}

#[cfg(feature = "grpc")]
impl RemoteWorker {
    pub fn new(node: WorkerNode) -> Self {
        Self { node }
    }

    pub fn node(&self) -> &WorkerNode {
        &self.node
    }
}

#[cfg(feature = "grpc")]
impl InferenceWorker for RemoteWorker {
    fn name(&self) -> &str {
        &self.node.id
    }

    fn run_batch(&self, batch: &[InferenceRequest]) -> Result<Vec<InferenceResponse>> {
        let endpoint = self.node.endpoint.clone();
        let batch = batch.to_vec();

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(async_run(&endpoint, batch))),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| LuminaError::Internal(format!("tokio rt: {e}")))?;
                rt.block_on(async_run(&endpoint, batch))
            }
        }
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
        Err(LuminaError::Inference(format!(
            "RemoteWorker '{}' needs `--features grpc`",
            self.node.id
        )))
    }
}
