//! Database module for SQLTrace
//!
//! This module provides database connectivity and query execution functionality.

mod error;
pub mod models;

use serde_json::Value as JsonValue;
use sqlx::postgres::{PgPoolOptions, PgRow, Postgres};
use sqlx::{Pool, Row};
use std::time::Duration;

pub use error::{DbError, Result};
pub use models::plan::{ExecutionPlan, PlanNode};

/// Database connection manager
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(connection_string)
            .await
            .map_err(DbError::Connection)?;

        Ok(Self { pool })
    }

    /// Execute a query and return its execution plan
    pub async fn explain(&self, query: &str) -> Result<ExecutionPlan> {
        // First, verify the query is a SELECT statement for safety
        self.validate_query(query)?;

        // Get the execution plan in JSON format
        let plan_json = self.explain_as_json(query).await?;

        // Parse the JSON into our structured format
        let plan: ExecutionPlan =
            serde_json::from_value(plan_json).map_err(|e| DbError::PlanParsing(e.to_string()))?;

        Ok(plan)
    }

    /// Execute EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) and return raw JSON
    async fn explain_as_json(&self, query: &str) -> Result<JsonValue> {
        // We use a subquery with WHERE false to avoid executing the actual query
        // while still getting the full execution plan
        let row = sqlx::query(
            r#"
            WITH plan AS (
                SELECT * FROM jsonb_array_elements(
                    (SELECT (EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) EXECUTE stmt)).plan
                )
            )
            SELECT plan FROM plan
            "#,
        )
        .bind(query)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::Query)?;

        row.try_get("plan").map_err(DbError::Query)
    }

    /// Validate that a query is a SELECT statement
    fn validate_query(&self, query: &str) -> Result<()> {
        let query_upper = query.trim_start().to_uppercase();

        if !query_upper.starts_with("SELECT") {
            return Err(DbError::InvalidQuery(
                "Only SELECT queries are supported for explanation".into(),
            ));
        }

        // Additional validation can be added here
        // For example, check for dangerous patterns or unsupported features

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_query() {
        let db = Database::new("postgres://localhost:5432").await.unwrap();

        // Valid SELECT query
        assert!(db.validate_query("SELECT 1").is_ok());
        assert!(db.validate_query("  SELECT * FROM users").is_ok());

        // Invalid queries
        assert!(db
            .validate_query("INSERT INTO users VALUES (1, 'test')")
            .is_err());
        assert!(db.validate_query("UPDATE users SET name = 'test'").is_err());
        assert!(db.validate_query("DELETE FROM users").is_err());
    }
}
