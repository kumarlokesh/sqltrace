//! PostgreSQL database engine implementation

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use super::{
    ConnectionConfig, DatabaseEngine, DatabaseFeature, DatabaseInfo, EngineError, EngineType,
    QueryCategory, SampleQuery,
};
use crate::db::models::ExecutionPlan;

/// PostgreSQL database engine implementation
#[derive(Debug)]
pub struct PostgreSQLEngine {
    pool: PgPool,
    #[allow(dead_code)]
    config: ConnectionConfig,
}

impl PostgreSQLEngine {
    /// Create a new PostgreSQL engine instance
    pub async fn new(config: ConnectionConfig) -> Result<Self, EngineError> {
        let pool = PgPool::connect(&config.connection_string)
            .await
            .map_err(|e| {
                EngineError::Connection(format!("Failed to connect to PostgreSQL: {}", e))
            })?;

        Ok(Self { pool, config })
    }
}

#[async_trait]
impl DatabaseEngine for PostgreSQLEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::PostgreSQL
    }

    async fn test_connection(&self) -> Result<bool, EngineError> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => Err(EngineError::Connection(format!(
                "Connection test failed: {}",
                e
            ))),
        }
    }

    async fn explain_query(&self, query: &str) -> Result<ExecutionPlan, EngineError> {
        // Use the existing explain functionality from our db module
        let explain_query = format!("EXPLAIN (FORMAT JSON, ANALYZE, BUFFERS) {}", query);

        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                EngineError::QueryExecution(format!("Failed to execute EXPLAIN: {}", e))
            })?;

        let explain_json: serde_json::Value = row.try_get(0).map_err(|e| {
            EngineError::PlanParsing(format!("Failed to get EXPLAIN result: {}", e))
        })?;

        // Parse the execution plan using existing logic
        crate::db::parse_execution_plan(&explain_json)
            .map_err(|e| EngineError::PlanParsing(format!("Failed to parse execution plan: {}", e)))
    }

    async fn validate_query(&self, query: &str) -> Result<(), EngineError> {
        // Use EXPLAIN without ANALYZE to validate syntax without executing
        let explain_query = format!("EXPLAIN {}", query);

        sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EngineError::QueryExecution(format!("Query validation failed: {}", e)))?;

        Ok(())
    }

    async fn get_version_info(&self) -> Result<DatabaseInfo, EngineError> {
        let version_row = sqlx::query("SELECT version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EngineError::Connection(format!("Failed to get version: {}", e)))?;

        let version: String = version_row
            .try_get(0)
            .map_err(|e| EngineError::Connection(format!("Failed to parse version: {}", e)))?;

        Ok(DatabaseInfo {
            engine_type: EngineType::PostgreSQL,
            version,
            connection_status: "Connected".to_string(),
            features_supported: vec![
                DatabaseFeature::DetailedExecutionPlan,
                DatabaseFeature::ActualRowCounts,
                DatabaseFeature::CostEstimation,
                DatabaseFeature::IndexSuggestions,
                DatabaseFeature::QueryOptimizationHints,
                DatabaseFeature::ParallelExecution,
                DatabaseFeature::PartitionedTables,
            ],
        })
    }

    fn get_sample_queries(&self) -> Vec<SampleQuery> {
        vec![
            SampleQuery {
                name: "Simple Select".to_string(),
                description: "Basic table scan with filtering".to_string(),
                query: "SELECT * FROM customers WHERE country = 'USA';".to_string(),
                category: QueryCategory::BasicSelect,
            },
            SampleQuery {
                name: "Inner Join".to_string(),
                description: "Join two tables with performance considerations".to_string(),
                query: "SELECT c.name, o.total FROM customers c JOIN orders o ON c.id = o.customer_id WHERE o.total > 100;".to_string(),
                category: QueryCategory::Join,
            },
            SampleQuery {
                name: "Complex Join".to_string(),
                description: "Multi-table join with aggregation".to_string(),
                query: "SELECT c.name, COUNT(o.id) as order_count, SUM(oi.quantity * p.price) as total_spent FROM customers c LEFT JOIN orders o ON c.id = o.customer_id LEFT JOIN order_items oi ON o.id = oi.order_id LEFT JOIN products p ON oi.product_id = p.id GROUP BY c.id, c.name HAVING SUM(oi.quantity * p.price) > 500 ORDER BY total_spent DESC;".to_string(),
                category: QueryCategory::Join,
            },
            SampleQuery {
                name: "Aggregation Query".to_string(),
                description: "Grouping and aggregation with sorting".to_string(),
                query: "SELECT category, COUNT(*) as product_count, AVG(price) as avg_price FROM products GROUP BY category ORDER BY avg_price DESC;".to_string(),
                category: QueryCategory::Aggregation,
            },
            SampleQuery {
                name: "Subquery Example".to_string(),
                description: "Correlated subquery performance analysis".to_string(),
                query: "SELECT * FROM products p WHERE p.price > (SELECT AVG(price) FROM products p2 WHERE p2.category = p.category);".to_string(),
                category: QueryCategory::Subquery,
            },
            SampleQuery {
                name: "Window Function".to_string(),
                description: "Window function with partitioning".to_string(),
                query: "SELECT name, price, category, ROW_NUMBER() OVER (PARTITION BY category ORDER BY price DESC) as price_rank FROM products;".to_string(),
                category: QueryCategory::Window,
            },
            SampleQuery {
                name: "Performance Test".to_string(),
                description: "Large table scan for performance testing".to_string(),
                query: "SELECT c.*, COUNT(o.id) FROM customers c LEFT JOIN orders o ON c.id = o.customer_id GROUP BY c.id;".to_string(),
                category: QueryCategory::Performance,
            },
        ]
    }

    fn supports_feature(&self, feature: &DatabaseFeature) -> bool {
        match feature {
            DatabaseFeature::DetailedExecutionPlan => true,
            DatabaseFeature::ActualRowCounts => true,
            DatabaseFeature::CostEstimation => true,
            DatabaseFeature::IndexSuggestions => true,
            DatabaseFeature::QueryOptimizationHints => true,
            DatabaseFeature::ParallelExecution => true,
            DatabaseFeature::PartitionedTables => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgresql_features() {
        let config = ConnectionConfig {
            engine_type: EngineType::PostgreSQL,
            connection_string: "postgres://test".to_string(),
            max_connections: None,
            timeout_seconds: None,
        };
    }

    #[test]
    fn test_sample_queries() {
        let config = ConnectionConfig {
            engine_type: EngineType::PostgreSQL,
            connection_string: "postgres://test".to_string(),
            max_connections: None,
            timeout_seconds: None,
        };
    }
}
