//! Error handling for the SQL Trace application.
//!
//! This module defines the main error type `SqlTraceError` used throughout the application,
//! along with convenient type aliases and conversion implementations.

use crate::db::error::DbError;
use thiserror::Error;

/// The main error type for the SQL Trace application.
///
/// This enum represents all possible errors that can occur during the execution
/// of the application, including database errors, I/O errors, and configuration issues.
#[derive(Error, Debug)]
pub enum SqlTraceError {
    /// An error that occurred during database operations.
    /// Contains a message describing the database error.
    #[error("Database error: {0}")]
    Database(String),

    /// An error that occurred during JSON serialization or deserialization.
    /// Wraps the underlying `serde_json::Error`.
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// An error that occurred during I/O operations.
    /// Wraps the underlying `std::io::Error`.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An error that occurred during configuration.
    /// Contains a message describing the configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// An error that occurred during execution plan processing.
    /// Contains a message describing the plan-related error.
    #[error("Execution plan error: {0}")]
    PlanError(String),

    /// An error that occurred due to an invalid query.
    /// Contains a message describing why the query is invalid.
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

/// A specialized `Result` type for SQL Trace operations.
///
/// This is a convenience type that defaults to using `SqlTraceError` as the error type.
pub type Result<T> = std::result::Result<T, SqlTraceError>;
