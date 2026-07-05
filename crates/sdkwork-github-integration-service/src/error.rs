use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("repository error: {0}")]
    Repository(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("integration error: {0}")]
    Integration(String),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("not found: {0}")]
    NotFound(String),
}
