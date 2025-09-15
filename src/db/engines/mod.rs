//! Database engine abstraction layer
//!
//! This module provides an abstract interface for different database engines,
//! allowing SQLTrace to support PostgreSQL, MySQL, and SQLite with a unified API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;

use crate::db::models::ExecutionPlan;

pub mod mysql;
pub mod postgresql;
pub mod sqlite;

/// Errors that can occur during database operations
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query execution error: {0}")]
    QueryExecution(String),

    #[error("Plan parsing error: {0}")]
    PlanParsing(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Database engine types supported by SQLTrace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngineType {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::PostgreSQL => write!(f, "PostgreSQL"),
            EngineType::MySQL => write!(f, "MySQL"),
            EngineType::SQLite => write!(f, "SQLite"),
        }
    }
}

/// Connection configuration for different database engines
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub engine_type: EngineType,
    pub connection_string: String,
    pub max_connections: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

/// Abstract trait for database engine implementations
#[async_trait]
pub trait DatabaseEngine: Send + Sync {
    /// Get the engine type
    fn engine_type(&self) -> EngineType;

    /// Test the database connection
    async fn test_connection(&self) -> Result<bool, EngineError>;

    /// Execute a query and return the execution plan
    async fn explain_query(&self, query: &str) -> Result<ExecutionPlan, EngineError>;

    /// Validate query syntax without executing it
    async fn validate_query(&self, query: &str) -> Result<(), EngineError>;

    /// Get database version and connection info
    async fn get_version_info(&self) -> Result<DatabaseInfo, EngineError>;

    /// Get available sample queries for this engine
    fn get_sample_queries(&self) -> Vec<SampleQuery>;

    /// Check if a specific feature is supported by this engine
    fn supports_feature(&self, feature: &DatabaseFeature) -> bool;
}

/// Database connection and version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub engine_type: EngineType,
    pub version: String,
    pub connection_status: String,
    pub features_supported: Vec<DatabaseFeature>,
}

/// Database features that may or may not be supported by different engines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseFeature {
    DetailedExecutionPlan,
    ActualRowCounts,
    CostEstimation,
    IndexSuggestions,
    QueryOptimizationHints,
    ParallelExecution,
    PartitionedTables,
}

/// Sample query for demonstration purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleQuery {
    pub name: String,
    pub description: String,
    pub query: String,
    pub category: QueryCategory,
}

/// Query category for sample queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryCategory {
    BasicSelect,
    Join,
    Aggregation,
    Subquery,
    CTE,
    Window,
    Performance,
}

/// Unified database engine enum that wraps all supported engines
pub enum DatabaseEngineImpl {
    PostgreSQL(postgresql::PostgreSQLEngine),
    MySQL(mysql::MySQLEngine),
    SQLite(sqlite::SQLiteEngine),
}

#[async_trait]
impl DatabaseEngine for DatabaseEngineImpl {
    fn engine_type(&self) -> EngineType {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.engine_type(),
            DatabaseEngineImpl::MySQL(engine) => engine.engine_type(),
            DatabaseEngineImpl::SQLite(engine) => engine.engine_type(),
        }
    }

    async fn test_connection(&self) -> Result<bool, EngineError> {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.test_connection().await,
            DatabaseEngineImpl::MySQL(engine) => engine.test_connection().await,
            DatabaseEngineImpl::SQLite(engine) => engine.test_connection().await,
        }
    }

    async fn explain_query(&self, query: &str) -> Result<ExecutionPlan, EngineError> {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.explain_query(query).await,
            DatabaseEngineImpl::MySQL(engine) => engine.explain_query(query).await,
            DatabaseEngineImpl::SQLite(engine) => engine.explain_query(query).await,
        }
    }

    async fn validate_query(&self, query: &str) -> Result<(), EngineError> {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.validate_query(query).await,
            DatabaseEngineImpl::MySQL(engine) => engine.validate_query(query).await,
            DatabaseEngineImpl::SQLite(engine) => engine.validate_query(query).await,
        }
    }

    async fn get_version_info(&self) -> Result<DatabaseInfo, EngineError> {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.get_version_info().await,
            DatabaseEngineImpl::MySQL(engine) => engine.get_version_info().await,
            DatabaseEngineImpl::SQLite(engine) => engine.get_version_info().await,
        }
    }

    fn get_sample_queries(&self) -> Vec<SampleQuery> {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.get_sample_queries(),
            DatabaseEngineImpl::MySQL(engine) => engine.get_sample_queries(),
            DatabaseEngineImpl::SQLite(engine) => engine.get_sample_queries(),
        }
    }

    fn supports_feature(&self, feature: &DatabaseFeature) -> bool {
        match self {
            DatabaseEngineImpl::PostgreSQL(engine) => engine.supports_feature(feature),
            DatabaseEngineImpl::MySQL(engine) => engine.supports_feature(feature),
            DatabaseEngineImpl::SQLite(engine) => engine.supports_feature(feature),
        }
    }
}

/// Factory for creating database engine instances
pub struct EngineFactory;

impl EngineFactory {
    /// Create a database engine instance based on the connection configuration
    pub async fn create_engine(
        config: ConnectionConfig,
    ) -> Result<DatabaseEngineImpl, EngineError> {
        match config.engine_type {
            EngineType::PostgreSQL => {
                let engine = postgresql::PostgreSQLEngine::new(config).await?;
                Ok(DatabaseEngineImpl::PostgreSQL(engine))
            }
            EngineType::MySQL => {
                let engine = mysql::MySQLEngine::new(config).await?;
                Ok(DatabaseEngineImpl::MySQL(engine))
            }
            EngineType::SQLite => {
                let engine = sqlite::SQLiteEngine::new(config).await?;
                Ok(DatabaseEngineImpl::SQLite(engine))
            }
        }
    }

    /// Detect engine type from connection string
    pub fn detect_engine_type(connection_string: &str) -> Result<EngineType, EngineError> {
        if connection_string.starts_with("postgres://")
            || connection_string.starts_with("postgresql://")
        {
            Ok(EngineType::PostgreSQL)
        } else if connection_string.starts_with("mysql://") {
            Ok(EngineType::MySQL)
        } else if connection_string.ends_with(".db")
            || connection_string.ends_with(".sqlite")
            || connection_string.starts_with("sqlite://")
        {
            Ok(EngineType::SQLite)
        } else {
            Err(EngineError::Configuration(
                "Unable to detect database engine type from connection string".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_type_detection() {
        assert_eq!(
            EngineFactory::detect_engine_type("postgres://user:pass@localhost/db").unwrap(),
            EngineType::PostgreSQL
        );

        assert_eq!(
            EngineFactory::detect_engine_type("mysql://user:pass@localhost/db").unwrap(),
            EngineType::MySQL
        );

        assert_eq!(
            EngineFactory::detect_engine_type("/path/to/database.sqlite").unwrap(),
            EngineType::SQLite
        );
    }

    #[test]
    fn test_engine_type_display() {
        assert_eq!(EngineType::PostgreSQL.to_string(), "PostgreSQL");
        assert_eq!(EngineType::MySQL.to_string(), "MySQL");
        assert_eq!(EngineType::SQLite.to_string(), "SQLite");
    }
}
