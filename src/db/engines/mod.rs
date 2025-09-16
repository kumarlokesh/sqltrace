//! Database engine abstraction layer
//!
//! This module provides an abstract interface for different database engines,
//! allowing SQLTrace to support PostgreSQL, MySQL, and SQLite with a unified API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::db::models::ExecutionPlan;

pub mod mysql;
pub mod postgresql;
pub mod sqlite;

/// Errors that can occur during database operations
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query execution errors
    #[error("Query execution error: {0}")]
    QueryExecution(String),

    /// Plan parsing errors
    #[error("Plan parsing error: {0}")]
    PlanParsing(String),

    /// Unsupported operation errors
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Database engine types supported by SQLTrace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineType {
    /// PostgreSQL database engine
    PostgreSQL,
    /// MySQL database engine
    MySQL,
    /// SQLite database engine
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

/// Connection configuration for database engines
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// The type of database engine
    pub engine_type: EngineType,
    /// Connection string for the database
    pub connection_string: String,
    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// Timeout for database operations in seconds
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

/// Information about a database connection and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    /// The type of database engine
    pub engine_type: EngineType,
    /// Database version string
    pub version: String,
    /// Current connection status
    pub connection_status: String,
    /// List of supported database features
    pub features_supported: Vec<DatabaseFeature>,
}

/// Database features that may be supported by different engines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseFeature {
    /// Support for detailed execution plans with timing and row counts
    DetailedExecutionPlan,
    /// Support for actual row counts in execution plans
    ActualRowCounts,
    /// Support for cost estimation in query planning
    CostEstimation,
    /// Support for index usage suggestions
    IndexSuggestions,
    /// Support for query optimization hints
    QueryOptimizationHints,
    /// Support for parallel query execution
    ParallelExecution,
    /// Support for partitioned tables
    PartitionedTables,
}

/// Sample query for demonstration purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleQuery {
    /// Display name for the query
    pub name: String,
    /// Description of what the query does
    pub description: String,
    /// The SQL query text
    pub query: String,
    /// Category of the query
    pub category: QueryCategory,
}

/// Categories for organizing sample queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryCategory {
    /// Basic SELECT queries
    BasicSelect,
    /// JOIN queries
    Join,
    /// Aggregation queries (GROUP BY, HAVING)
    Aggregation,
    /// Subquery examples
    Subquery,
    /// Common Table Expression (CTE) examples
    CTE,
    /// Window function examples
    Window,
    /// Performance-focused queries
    Performance,
}

/// Enum wrapper for different database engine implementations
#[derive(Debug)]
pub enum DatabaseEngineImpl {
    /// PostgreSQL engine implementation
    PostgreSQL(postgresql::PostgreSQLEngine),
    /// MySQL engine implementation
    MySQL(mysql::MySQLEngine),
    /// SQLite engine implementation
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
