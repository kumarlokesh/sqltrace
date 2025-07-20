use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlTraceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Execution plan error: {0}")]
    PlanError(String),
}

pub type Result<T> = std::result::Result<T, SqlTraceError>;
