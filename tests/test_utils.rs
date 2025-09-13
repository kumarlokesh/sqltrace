//! Test utilities for integration tests

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static INIT: Once = Once::new();

/// Creates a connection to the default postgres database for administrative tasks
async fn create_admin_pool() -> PgPool {
    let admin_url = "postgres://postgres:postgres@localhost:5432/postgres";

    let mut retries = 5;
    loop {
        match PgPoolOptions::new()
            .max_connections(1)
            .connect(admin_url)
            .await
        {
            Ok(pool) => return pool,
            Err(e) if retries > 0 => {
                eprintln!(
                    "Failed to connect to admin database: {}. Retrying... ({} attempts left)",
                    e, retries
                );
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => panic!("Failed to connect to admin database: {}", e),
        }
    }
}

/// Creates a new test database with a unique name
async fn create_test_database() -> (String, PgPool) {
    let admin_pool = create_admin_pool().await;
    let db_name = format!(
        "test_db_{}_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        Uuid::new_v4()
    );

    // Create a new database
    sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
        .execute(&admin_pool)
        .await
        .expect("Failed to create test database");

    // Connect to the new database
    let test_db_url = format!("postgres://postgres:postgres@localhost:5432/{}", db_name);
    let test_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");

    (db_name, test_pool)
}

/// Drops a test database
async fn drop_test_database(db_name: &str) {
    let admin_pool = create_admin_pool().await;

    // Terminate all connections to the test database
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity
         WHERE datname = $1 AND pid <> pg_backend_pid()",
    ))
    .bind(&db_name)
    .execute(&admin_pool)
    .await
    .ok();

    // Drop the database
    sqlx::query(&format!("DROP DATABASE IF EXISTS \"{}\"", db_name))
        .execute(&admin_pool)
        .await
        .ok();
}

/// Creates a connection pool for testing using a fresh database
pub async fn create_test_pool() -> (String, PgPool) {
    // Load environment variables from .env file if it exists
    dotenv::from_filename(".env").ok();
    dotenv::from_filename("tests/test.env").ok();
    dotenv::dotenv().ok();

    create_test_database().await
}

/// Gets a test database URL for testing
pub fn get_test_database_url() -> String {
    dotenv::from_filename(".env").ok();
    dotenv::from_filename("tests/test.env").ok();
    dotenv::dotenv().ok();

    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string())
}

/// Sets up test tables and data
pub async fn setup_test_database(pool: &PgPool) -> sqlx::Result<()> {
    // Create test tables
    let create_tables = [
        // Create users table
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )",
        // Create posts table
        "CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            user_id INTEGER REFERENCES users(id),
            title VARCHAR(200) NOT NULL,
            content TEXT,
            published BOOLEAN DEFAULT false,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )",
        // Create indexes for testing
        "CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(published) WHERE published = true",
    ];

    // Execute each statement separately
    for sql in &create_tables {
        sqlx::query(sql).execute(pool).await?;
    }

    // Insert test data if tables are empty
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if user_count.0 == 0 {
        // Insert test users - one at a time
        let user_inserts = [
            "INSERT INTO users (name, email) VALUES ('Test User 1', 'test1@example.com')",
            "INSERT INTO users (name, email) VALUES ('Test User 2', 'test2@example.com')",
            "INSERT INTO users (name, email) VALUES ('Test User 3', 'test3@example.com')",
        ];

        for sql in &user_inserts {
            sqlx::query(sql).execute(pool).await?;
        }

        // Insert test posts - one at a time
        let post_inserts = [
            "INSERT INTO posts (user_id, title, content, published) VALUES (1, 'First Post', 'This is the first post', true)",
            "INSERT INTO posts (user_id, title, content, published) VALUES (1, 'Draft Post', 'This is a draft', false)",
            "INSERT INTO posts (user_id, title, content, published) VALUES (2, 'Second User Post', 'Content from second user', true)",
        ];

        for sql in &post_inserts {
            sqlx::query(sql).execute(pool).await?;
        }
    }

    Ok(())
}

/// Drops all test tables and resets the database to a clean state
pub async fn teardown_test_database(pool: &PgPool) -> sqlx::Result<()> {
    // Disable triggers to avoid dependency issues during cleanup
    sqlx::query("SET session_replication_role = 'replica'")
        .execute(pool)
        .await?;

    // Get all tables in the public schema
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables
         WHERE schemaname = 'public'",
    )
    .fetch_all(pool)
    .await?;

    // Drop all tables with CASCADE
    for table in tables {
        let sql = format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table);
        if let Err(e) = sqlx::query(&sql).execute(pool).await {
            eprintln!("Warning: Failed to drop table '{}': {}", table, e);
        }
    }

    // Get all sequences in the public schema
    let sequences: Vec<String> = sqlx::query_scalar(
        "SELECT sequence_name FROM information_schema.sequences
         WHERE sequence_schema = 'public'",
    )
    .fetch_all(pool)
    .await?;

    // Drop all sequences
    for seq in sequences {
        let sql = format!("DROP SEQUENCE IF EXISTS \"{}\" CASCADE", seq);
        if let Err(e) = sqlx::query(&sql).execute(pool).await {
            eprintln!("Warning: Failed to drop sequence '{}': {}", seq, e);
        }
    }

    // Re-enable triggers
    sqlx::query("SET session_replication_role = 'origin'")
        .execute(pool)
        .await?;

    Ok(())
}

/// Helper to run a test with a fresh database
pub async fn with_test_database<F, Fut>(test: F) -> anyhow::Result<()>
where
    F: FnOnce(PgPool) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    // Create a temporary database for testing
    let (db_name, pool) = create_test_pool().await;

    // Set up test data
    setup_test_database(&pool).await?;

    // Run the test with the pool
    let test_result = test(pool).await;

    // Clean up by dropping the test database
    drop_test_database(&db_name).await;

    test_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn test_database_setup() -> anyhow::Result<()> {
        with_test_database(|pool| async move {
            // Test that users were inserted
            let user_count: i64 = sqlx::query("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await?
                .get(0);

            assert!(user_count > 0, "Expected users to be inserted");

            // Test that posts were inserted
            let post_count: i64 = sqlx::query("SELECT COUNT(*) FROM posts")
                .fetch_one(&pool)
                .await?
                .get(0);

            assert!(post_count > 0, "Expected posts to be inserted");

            Ok(())
        })
        .await
    }
}
