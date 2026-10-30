use thiserror::Error;

pub type Result<T> = std::result::Result<T, LuminaError>;

#[derive(Debug, Error)]
pub enum LuminaError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("worker unavailable: {0}")]
    WorkerUnavailable(String),

    #[error("batch overflow (max {max})")]
    BatchOverflow { max: usize },

    #[error("inference failed: {0}")]
    Inference(String),

    #[error("internal error: {0}")]
    Internal(String),
}
