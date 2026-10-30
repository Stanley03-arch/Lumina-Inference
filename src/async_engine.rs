use crate::batch::{InferenceRequest, InferenceResponse};
use crate::config::EngineConfig;
use crate::engine::InferenceEngine;
use crate::error::Result;
use crate::metrics::MetricsSnapshot;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Instant};

enum Cmd {
    Submit {
        req: InferenceRequest,
        reply: oneshot::Sender<Result<Vec<InferenceResponse>>>,
    },
    Metrics {
        reply: oneshot::Sender<MetricsSnapshot>,
    },
    Shutdown,
}

/// Async façade: clients submit via channel; idle batches flush after timeout.
pub struct AsyncInferenceEngine {
    tx: mpsc::Sender<Cmd>,
}

impl AsyncInferenceEngine {
    pub fn spawn(cfg: EngineConfig) -> Result<Self> {
        let batch_timeout = Duration::from_millis(cfg.batch_timeout_ms.max(1));
        let mut engine = InferenceEngine::new(cfg)?;
        let (tx, mut rx) = mpsc::channel::<Cmd>(256);

        tokio::spawn(async move {
            let mut pending: Vec<oneshot::Sender<Result<Vec<InferenceResponse>>>> = Vec::new();
            let mut deadline: Option<Instant> = None;

            loop {
                let wait = deadline.map(|d| d.saturating_duration_since(Instant::now()));

                let cmd = if let Some(w) = wait {
                    match timeout(w, rx.recv()).await {
                        Ok(Some(c)) => Some(c),
                        Ok(None) => break,
                        Err(_) => {
                            // Timeout: flush partial batch
                            if !pending.is_empty() {
                                let result = engine.flush();
                                for r in pending.drain(..) {
                                    let _ = r.send(result.clone());
                                }
                            }
                            deadline = None;
                            continue;
                        }
                    }
                } else {
                    rx.recv().await
                };

                match cmd {
                    Some(Cmd::Submit { req, reply }) => {
                        match engine.submit(req) {
                            Ok(Some(batch)) => {
                                let _ = reply.send(Ok(batch));
                                // Also complete any waiters that shared the same flush
                                for r in pending.drain(..) {
                                    let _ = r.send(Ok(Vec::new()));
                                }
                                deadline = None;
                            }
                            Ok(None) => {
                                pending.push(reply);
                                if deadline.is_none() {
                                    deadline = Some(Instant::now() + batch_timeout);
                                }
                            }
                            Err(e) => {
                                let _ = reply.send(Err(e));
                            }
                        }
                    }
                    Some(Cmd::Metrics { reply }) => {
                        let _ = reply.send(engine.metrics());
                    }
                    Some(Cmd::Shutdown) | None => {
                        if !pending.is_empty() {
                            let result = engine.flush();
                            for r in pending.drain(..) {
                                let _ = r.send(result.clone());
                            }
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    pub async fn submit(&self, req: InferenceRequest) -> Result<Vec<InferenceResponse>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Cmd::Submit { req, reply })
            .await
            .map_err(|_| crate::LuminaError::Internal("engine channel closed".into()))?;
        rx.await
            .map_err(|_| crate::LuminaError::Internal("engine dropped reply".into()))?
    }

    pub async fn metrics(&self) -> Result<MetricsSnapshot> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Cmd::Metrics { reply })
            .await
            .map_err(|_| crate::LuminaError::Internal("engine channel closed".into()))?;
        rx.await
            .map_err(|_| crate::LuminaError::Internal("engine dropped reply".into()))
    }

    pub async fn shutdown(&self) {
        let _ = self.tx.send(Cmd::Shutdown).await;
    }
}
