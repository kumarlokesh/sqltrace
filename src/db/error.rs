//! Database error types for SQL Trace
//!
//! This module defines the error types used throughout the database module.

use serde_json::Error as JsonError;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use thiserror::Error;

/// Represents errors that can occur during database operations.
#[derive(Error, Debug)]
pub enum DbError {
    /// Failed to establish a database connection
    #[error("Database connection error: {0}")]
    Connection(String),

    /// Error occurred while executing a query
    #[error("Query execution error: {0}")]
    Query(String),

    /// Error occurred during JSON serialization/deserialization
    #[error("JSON parsing error: {0}")]
    Json(#[from] JsonError),

    /// I/O related error
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    /// Configuration related error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error occurred while parsing an execution plan
    #[error("Execution plan error: {0}")]
    PlanError(String),

    /// The provided SQL query is invalid
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

impl From<SqlxError> for DbError {
    /// Converts a SQLx error into a database error
    fn from(err: SqlxError) -> Self {
        match err {
            SqlxError::Io(io_err) => DbError::Io(io_err),
            SqlxError::Configuration(config_err) => DbError::Config(config_err.to_string()),
            _ => DbError::Query(err.to_string()),
        }
    }
}

// The From<serde_json::Error> is automatically derived by thiserror's #[error("...")] attribute

/// Convenience type for Results that use DbError
///
/// This is the standard result type returned by database operations.
pub type Result<T> = std::result::Result<T, DbError>;
