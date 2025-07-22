use crate::db::error::DbError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlTraceError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Execution plan error: {0}")]
    PlanError(String),

    #[error("Query error: {0}")]
    InvalidQuery(String),
}

impl From<sqlx::Error> for SqlTraceError {
    fn from(err: sqlx::Error) -> Self {
        SqlTraceError::Database(err.to_string())
    }
}

impl From<DbError> for SqlTraceError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Connection(msg) => SqlTraceError::Database(msg),
            DbError::Query(msg) => SqlTraceError::Database(msg),
            DbError::Json(e) => SqlTraceError::Json(e),
            DbError::Io(e) => SqlTraceError::Io(e),
            DbError::Config(msg) => SqlTraceError::Config(msg),
            DbError::PlanParsing(msg) => SqlTraceError::PlanError(msg),
            DbError::InvalidQuery(msg) => SqlTraceError::InvalidQuery(msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, SqlTraceError>;
