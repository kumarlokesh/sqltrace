//! Database module for SQL Trace
//!
//! This module handles all database interactions including connection management,
//! query execution, and plan analysis.

#![allow(dead_code)]

pub mod error;
pub mod models;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use std::time::Duration;

use crate::db::error::DbError;
use crate::db::models::plan::ExecutionPlan;
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

        // Execute the EXPLAIN query directly
        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e: sqlx::Error| DbError::Query(e.to_string()))
            .map_err(SqlTraceError::from)?;

        // The result is a single column containing the JSON plan
        let plan_json: serde_json::Value = row
            .try_get("QUERY PLAN")
            .map_err(|e: sqlx::Error| DbError::Query(e.to_string()))
            .map_err(SqlTraceError::from)?;

        // PostgreSQL EXPLAIN output is an array of objects, take the first one
        match plan_json.as_array() {
            Some(arr) if !arr.is_empty() => {
                let first_item = &arr[0];

                // Extract the plan object which contains the execution details
                if let Some(plan_obj) = first_item.get("Plan") {
                    // Create a properly formatted response object that matches the UI's expectations
                    let mut response = serde_json::Map::new();
                    response.insert("root".to_string(), plan_obj.clone());

                    // Add planning and execution times
                    if let Some(planning_time) = first_item.get("Planning Time") {
                        response.insert("planning_time".to_string(), planning_time.clone());
                    }

                    if let Some(execution_time) = first_item.get("Execution Time") {
                        response.insert("execution_time".to_string(), execution_time.clone());
                    }

                    // Parse the response into an ExecutionPlan
                    let exec_plan: ExecutionPlan = serde_json::from_value(
                        serde_json::Value::Object(response),
                    )
                    .map_err(|e| {
                        let err_msg = format!("Failed to format execution plan: {}", e);
                        SqlTraceError::from(DbError::PlanParsing(err_msg))
                    })?;

                    Ok(exec_plan)
                } else {
                    // Check if there's an error message
                    if let Some(error_msg) = first_item.get("error").and_then(|e| e.as_str()) {
                        return Err(SqlTraceError::from(DbError::PlanParsing(format!(
                            "Database error: {}",
                            error_msg
                        ))));
                    }

                    Err(SqlTraceError::from(DbError::PlanParsing(
                        "No 'Plan' field in EXPLAIN output".to_string(),
                    )))
                }
            }
            _ => {
                // Check if it's an error object
                if let Some(error_msg) = plan_json.get("error").and_then(|e| e.as_str()) {
                    return Err(SqlTraceError::from(DbError::PlanParsing(format!(
                        "Database error: {}",
                        error_msg
                    ))));
                }

                Err(SqlTraceError::from(DbError::PlanParsing(
                    "Unexpected EXPLAIN output format".to_string(),
                )))
            }
        }
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
