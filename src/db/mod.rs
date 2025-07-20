use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;

use crate::error::{Result, SqlTraceError};

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(connection_string)
            .await?;

        Ok(Self { pool })
    }

    pub async fn explain_query(&self, query: &str) -> Result<serde_json::Value> {
        // Use EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) to get detailed execution plan
        let row: (serde_json::Value,) = sqlx::query_as(
            "SELECT jsonb_path_query_first(explain_json, '$[0]') FROM (SELECT EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) AS explain_json FROM (SELECT 1) AS _ WHERE 1=0) _"
        )
        .bind(query)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }
}
