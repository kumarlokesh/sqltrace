//! Integration tests with real PostgreSQL database

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqltrace_rs::db::Database;
use std::env;
use tower::ServiceExt;

mod test_utils;

/// Get the test database URL from environment
fn get_database_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/sqltrace_dev".to_string())
}

/// Create a test app with real database connection
async fn create_app() -> Router {
    let db_url = get_database_url();
    let db = Database::new(&db_url)
        .await
        .expect("Failed to connect to database - ensure PostgreSQL is running");

    // Use the actual router from main
    let state = sqltrace_rs::AppState { db };
    sqltrace_rs::create_router(state)
}

/// Helper to make HTTP requests
async fn make_request(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = if let Some(body) = body {
        Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap()
    } else {
        Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap()
    };

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body)
        .unwrap_or_else(|_| json!({"text": String::from_utf8_lossy(&body)}));

    (status, json)
}

#[tokio::test]
async fn test_database_connection() {
    let db_url = get_database_url();
    let db = Database::new(&db_url).await;
    assert!(
        db.is_ok(),
        "Should be able to connect to PostgreSQL database"
    );
}

#[tokio::test]
async fn test_health_endpoint_integration() {
    let app = create_app().await;
    let (status, body) = make_request(&app, "GET", "/api/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_simple_select_query() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT 1 as test_value"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_null(),
        "Query should not return an error: {}",
        body["error"]
    );
    assert!(body["plan"].is_object(), "Should return a plan object");

    let plan = &body["plan"];
    assert!(plan["nodes"].is_array(), "Plan should have nodes array");
    assert!(
        plan["root_indices"].is_array(),
        "Plan should have root_indices array"
    );
}

#[tokio::test]
async fn test_table_scan_query() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT * FROM ecommerce.users LIMIT 5"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_null(),
        "Query should not return an error: {}",
        body["error"]
    );

    let plan = &body["plan"];
    assert!(plan["nodes"].is_array());
    let nodes = plan["nodes"].as_array().unwrap();
    assert!(!nodes.is_empty(), "Should have at least one plan node");
}

#[tokio::test]
async fn test_join_query() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT u.username, o.id FROM ecommerce.users u LEFT JOIN ecommerce.orders o ON u.id = o.user_id LIMIT 10"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_null(),
        "Query should not return an error: {}",
        body["error"]
    );

    let plan = &body["plan"];
    let nodes = plan["nodes"].as_array().unwrap();
    assert!(!nodes.is_empty(), "Should have plan nodes for join query");

    // Check that we have meaningful node data
    let first_node = &nodes[0];
    assert!(first_node["node_type"].is_string(), "Should have node_type");
    assert!(
        first_node["total_cost"].is_number(),
        "Should have total_cost"
    );
}

#[tokio::test]
async fn test_aggregation_query() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT COUNT(*) as user_count FROM ecommerce.users"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_null(),
        "Query should not return an error: {}",
        body["error"]
    );

    let plan = &body["plan"];
    let nodes = plan["nodes"].as_array().unwrap();
    assert!(
        !nodes.is_empty(),
        "Should have plan nodes for aggregation query"
    );
}

#[tokio::test]
async fn test_complex_join_aggregation() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT u.username, COUNT(o.id) as order_count FROM ecommerce.users u LEFT JOIN ecommerce.orders o ON u.id = o.user_id GROUP BY u.id, u.username ORDER BY order_count DESC LIMIT 10"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_null(),
        "Query should not return an error: {}",
        body["error"]
    );

    let plan = &body["plan"];
    let nodes = plan["nodes"].as_array().unwrap();
    assert!(
        nodes.len() > 1,
        "Complex query should have multiple plan nodes"
    );

    // Verify we have some expected operations in the plan
    let node_types: Vec<String> = nodes
        .iter()
        .map(|n| n["node_type"].as_str().unwrap_or("").to_string())
        .collect();

    println!("Node types in complex query: {:?}", node_types);
    assert!(!node_types.is_empty(), "Should have node types in the plan");
}

#[tokio::test]
async fn test_invalid_table_query() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT * FROM nonexistent_table"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_string(),
        "Should return an error for non-existent table"
    );

    let error_msg = body["error"].as_str().unwrap();
    assert!(
        error_msg.contains("does not exist") || error_msg.contains("relation"),
        "Error should mention table doesn't exist: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_syntax_error_query() {
    let app = create_app().await;

    let query = json!({
        "query": "SELECT FROM WHERE"
    });

    let (status, body) = make_request(&app, "POST", "/api/explain", Some(query)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["error"].is_string(),
        "Should return an error for syntax error"
    );
}

#[tokio::test]
async fn test_serve_static_files() {
    let app = create_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("SQLTrace"),
        "Should contain SQLTrace in the HTML"
    );
    assert!(html.contains("<!DOCTYPE html>"), "Should be valid HTML");
}
