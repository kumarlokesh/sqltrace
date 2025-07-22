//! Database module for SQLTrace
//!
//! This module provides database connectivity and query execution functionality.

pub mod error;
pub mod models;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use std::time::Duration;

use crate::db::error::DbError;
use crate::db::models::plan::{ExecutionPlan, ExplainOutput};
use crate::error::{Result, SqlTraceError};

/// Database connection manager
#[derive(Debug, Clone)]
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
            .map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Create a new Database instance from an existing connection pool
    pub fn from_pool(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Execute a query and return its execution plan
    pub async fn explain(&self, query: &str) -> Result<ExecutionPlan> {
        // First validate the query
        self.validate_query(query)?;

        // Execute EXPLAIN ANALYZE with JSON output
        // Note: We need to use a raw query here because PostgreSQL doesn't support parameters in EXPLAIN
        let explain_query = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {}", query);

        println!("Executing EXPLAIN query: {}", explain_query);

        // Execute the EXPLAIN query directly
        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e: sqlx::Error| DbError::Query(e.to_string()))
            .map_err(|e| SqlTraceError::from(e))?;

        // The result is a single column containing the JSON plan
        let plan_json: serde_json::Value = row
            .try_get("QUERY PLAN")
            .map_err(|e: sqlx::Error| DbError::Query(e.to_string()))
            .map_err(|e| SqlTraceError::from(e))?;

        println!("Raw EXPLAIN JSON output: {:#?}", plan_json);

        // EXPLAIN (FORMAT JSON) returns an array with a single plan object
        // We need to extract the first element of the array
        let explain_output: ExplainOutput = match plan_json.as_array() {
            Some(arr) if !arr.is_empty() => {
                // Deserialize the first element of the array into ExplainOutput
                serde_json::from_value(arr[0].clone())
                    .map_err(|e| {
                        DbError::PlanParsing(format!("Failed to parse EXPLAIN output: {}", e))
                    })
                    .map_err(|e| SqlTraceError::from(e))?
            }
            _ => {
                return Err(DbError::PlanParsing(
                    "Expected non-empty array in EXPLAIN output".to_string(),
                ))
                .map_err(|e| SqlTraceError::from(e))
            }
        };

        // Convert the ExplainOutput to our ExecutionPlan
        Ok(ExecutionPlan {
            root: explain_output.plan,
            planning_time: explain_output.planning_time,
            execution_time: explain_output.execution_time,
        })
    }

    /// Validate that a query is a SELECT query
    fn validate_query(&self, query: &str) -> Result<()> {
        let query = query.trim().to_lowercase();

        // Check if the query starts with SELECT
        if !query.starts_with("select") {
            return Err(DbError::InvalidQuery(
                "Only SELECT queries are supported for explanation".to_string(),
            )
            .into());
        }

        // Check for forbidden SQL keywords
        let forbidden = [
            "insert ", "update ", "delete ", "drop ", "create ", "alter ",
        ];
        for keyword in &forbidden {
            if query.contains(keyword) {
                return Err(DbError::InvalidQuery(format!(
                    "Query contains forbidden keyword: {}",
                    keyword.trim()
                ))
                .into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn get_test_db() -> Database {
        // Load environment variables from .env file if it exists
        dotenv::from_filename(".env").ok();
        // Then load from tests/test.env if it exists
        dotenv::from_filename("tests/test.env").ok();
        // Finally, load from environment
        dotenv::dotenv().ok();

        // Default to Docker Compose configuration if not set
        let database_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/sqltrace_test".to_string()
        });

        println!("Connecting to database: {}", database_url);

        // Try to connect with retries to handle database startup time
        let mut retries = 5;
        loop {
            match Database::new(&database_url).await {
                Ok(db) => {
                    println!("Successfully connected to test database");
                    return db;
                }
                Err(e) if retries > 0 => {
                    eprintln!(
                        "Failed to connect to test database: {}. Retrying... ({} attempts left)",
                        e, retries
                    );
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    panic!(
                        "Failed to connect to test database after multiple attempts: {}",
                        e
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_validate_query() {
        let db = get_test_db().await;

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
