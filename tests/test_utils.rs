//! Test utilities for integration tests

use sqlx::PgPool;
use std::env;

/// Creates a connection pool for testing using Docker Compose configuration
pub async fn create_test_pool() -> PgPool {
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
        match PgPool::connect(&database_url).await {
            Ok(pool) => {
                println!("Successfully connected to test database");
                return pool;
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

/// Drops all test tables
pub async fn teardown_test_database(pool: &PgPool) -> sqlx::Result<()> {
    let drop_statements = [
        "DROP TABLE IF EXISTS posts CASCADE",
        "DROP TABLE IF EXISTS users CASCADE",
    ];

    // Execute each DROP statement separately
    for sql in &drop_statements {
        if let Err(e) = sqlx::query(sql).execute(pool).await {
            eprintln!("Warning: Failed to execute '{}': {}", sql, e);
            // Continue with other statements even if one fails
        }
    }

    Ok(())
}

/// Helper to run a test with a fresh database
pub async fn with_test_database<F, Fut>(test: F) -> anyhow::Result<()>
where
    F: FnOnce(PgPool) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    // Create a temporary database for testing
    let pool = create_test_pool().await;

    // Set up test data
    setup_test_database(&pool).await?;

    // Clone the pool for cleanup
    let pool_for_cleanup = pool.clone();

    // Run the test with the original pool
    let result = test(pool).await;

    // Clean up using the cloned pool
    teardown_test_database(&pool_for_cleanup).await?;

    result
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
