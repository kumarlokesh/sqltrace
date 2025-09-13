//! Web server setup and configuration

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use crate::db::Database;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db: Database,
}

/// Request payload for the explain endpoint
#[derive(Deserialize)]
struct ExplainRequest {
    query: String,
}

/// Response payload for the explain endpoint
#[derive(Serialize)]
struct ExplainResponse {
    plan: serde_json::Value,
    error: Option<String>,
}

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/api/explain", post(explain_handler))
        .route("/api/health", get(health_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

/// Serve the main index.html file
async fn serve_index() -> Html<String> {
    let html = tokio::fs::read_to_string("static/index.html")
        .await
        .unwrap_or_else(|_| {
            r#"
            <!DOCTYPE html>
            <html>
            <head><title>SQLTrace - Error</title></head>
            <body>
                <h1>SQLTrace</h1>
                <p>Error: Could not load index.html. Make sure the static files are present.</p>
            </body>
            </html>
            "#
            .to_string()
        });

    Html(html)
}

/// Health check endpoint
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "sqltrace-rs"
    }))
}

/// Handle SQL query explanation requests
async fn explain_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExplainRequest>,
) -> Result<Json<ExplainResponse>, StatusCode> {
    // Validate the query syntax first
    if let Err(validation_error) = crate::web::validate_query(&payload.query) {
        return Ok(Json(ExplainResponse {
            plan: serde_json::json!({}),
            error: Some(validation_error),
        }));
    }

    // Execute the query and get the execution plan
    match state.db.explain(&payload.query).await {
        Ok(plan) => {
            // Convert the plan to the UI format for the frontend
            let plan_tree = crate::ui::plan_to_web_format(&plan);
            match serde_json::to_value(plan_tree) {
                Ok(plan_value) => Ok(Json(ExplainResponse {
                    plan: plan_value,
                    error: None,
                })),
                Err(e) => Ok(Json(ExplainResponse {
                    plan: serde_json::json!({}),
                    error: Some(format!("Failed to serialize execution plan: {}", e)),
                })),
            }
        }
        Err(e) => Ok(Json(ExplainResponse {
            plan: serde_json::json!({}),
            error: Some(e.to_string()),
        })),
    }
}
