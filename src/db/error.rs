use serde_json::Error as JsonError;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Query execution error: {0}")]
    Query(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] JsonError),

    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Execution plan error: {0}")]
    PlanParsing(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

impl From<SqlxError> for DbError {
    fn from(err: SqlxError) -> Self {
        match err {
            SqlxError::PoolTimedOut | SqlxError::PoolClosed | SqlxError::PoolTimedOut => {
                DbError::Connection(err.to_string())
            }
            _ => DbError::Query(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
