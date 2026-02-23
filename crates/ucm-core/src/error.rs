use thiserror::Error;

#[derive(Error, Debug)]
pub enum UcmError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Edge not found between {from} and {to}")]
    EdgeNotFound { from: String, to: String },

    #[error("Duplicate entity: {0}")]
    DuplicateEntity(String),

    #[error("Invalid SCIP identifier: {0}")]
    InvalidScipId(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Ingestion error: {0}")]
    Ingestion(String),
}

pub type Result<T> = std::result::Result<T, UcmError>;
