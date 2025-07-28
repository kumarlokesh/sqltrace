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
use crate::db::models::plan::{ExecutionPlan, ExplainPlan, PlanNode};
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

        // Log the raw plan JSON to a file for debugging
        let debug_path = "debug_plan.json";
        let json_str = serde_json::to_string_pretty(&plan_json)
            .unwrap_or_else(|_| "<invalid JSON>".to_string());

        if let Err(e) = std::fs::write(debug_path, &json_str) {
            eprintln!("Failed to write debug plan to {}: {}", debug_path, e);
        } else {
            eprintln!("Wrote debug plan to {}", debug_path);
            eprintln!(
                "Raw plan JSON (first 1000 chars): {}",
                &json_str.chars().take(1000).collect::<String>()
            );
        }

        // First, try to parse as an array of plans (the common case)
        let explain_plan = if let Ok(mut explain_outputs) =
            serde_json::from_value::<Vec<ExplainPlan>>(plan_json.clone())
        {
            explain_outputs
                .into_iter()
                .next()
                .ok_or_else(|| DbError::PlanError("Empty plan array".to_string()))?
        }
        // If that fails, try to parse as a single plan object (shouldn't happen with current PostgreSQL versions)
        else if let Ok(explain_plan) = serde_json::from_value::<ExplainPlan>(plan_json.clone()) {
            explain_plan
        }
        // If both fail, try to parse the plan field directly (for backward compatibility)
        else if let Some(plan_obj) = plan_json.get("Plan").and_then(|p| p.as_object()) {
            let plan =
                serde_json::from_value::<PlanNode>(serde_json::Value::Object(plan_obj.clone()))
                    .map_err(|e| DbError::PlanError(format!("Failed to parse plan: {}", e)))?;

            ExplainPlan {
                plan,
                planning_time: plan_json
                    .get("Planning Time")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                execution_time: plan_json
                    .get("Execution Time")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            }
        }
        // If all else fails, return a descriptive error with the actual JSON for debugging
        else {
            let json_str = serde_json::to_string_pretty(&plan_json)
                .unwrap_or_else(|_| "<invalid JSON>".to_string());
            return Err(DbError::PlanError(format!(
                "Failed to parse execution plan. Expected format: [{{Plan: ..., 'Planning Time': ..., 'Execution Time': ...}}]. Got: {}",
                json_str
            )).into());
        };

        // Convert to our internal ExecutionPlan format
        Ok(ExecutionPlan {
            root: explain_plan.plan,
            planning_time: explain_plan.planning_time,
            execution_time: explain_plan.execution_time,
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

        // Try to connect with retries to handle database startup time
        let mut retries = 5;
        loop {
            match Database::new(&database_url).await {
                Ok(db) => return db,
                Err(_e) if retries > 0 => {
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
