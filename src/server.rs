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

use crate::advisor::QueryAdvisor;
use crate::benchmark::{BenchmarkConfig, BenchmarkResult, BenchmarkSuite};
use crate::db::Database;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db: Database,
    /// Query optimization advisor
    pub advisor: QueryAdvisor,
}

/// Request payload for the explain endpoint
#[derive(Deserialize)]
struct ExplainRequest {
    query: String,
}

/// Response payload for the explain endpoint
#[derive(Serialize)]
struct ExplainResponse {
    plan: Option<serde_json::Value>,
    error: Option<String>,
    advisor_analysis: Option<crate::advisor::AdvisorAnalysis>,
}

/// Request payload for the benchmark endpoint
#[derive(Deserialize)]
struct BenchmarkRequest {
    query: String,
    config: Option<BenchmarkConfig>,
}

/// Response payload for the benchmark endpoint
#[derive(Serialize)]
struct BenchmarkResponse {
    result: Option<BenchmarkResult>,
    error: Option<String>,
}

/// Request payload for benchmark comparison
#[derive(Deserialize)]
struct BenchmarkCompareRequest {
    query_a: String,
    query_b: String,
    label_a: String,
    label_b: String,
    config: Option<BenchmarkConfig>,
}

/// Response payload for benchmark comparison
#[derive(Serialize)]
struct BenchmarkCompareResponse {
    comparison: Option<crate::benchmark::BenchmarkComparison>,
    error: Option<String>,
}

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/api/explain", post(explain_handler))
        .route("/api/health", get(health_handler))
        .route("/api/benchmark", post(benchmark_handler))
        .route("/api/benchmark/compare", post(benchmark_compare_handler))
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
            plan: Some(serde_json::json!({})),
            error: Some(validation_error),
            advisor_analysis: None,
        }));
    }

    // Execute the query and get the execution plan
    match state.db.explain(&payload.query).await {
        Ok(plan) => {
            // Run advisor analysis
            let advisor_analysis = state.advisor.analyze_plan(&plan);

            // Convert the plan to the UI format for the frontend
            let plan_tree = crate::ui::plan_to_web_format(&plan);
            match serde_json::to_value(plan_tree) {
                Ok(plan_value) => Ok(Json(ExplainResponse {
                    plan: Some(plan_value),
                    error: None,
                    advisor_analysis: Some(advisor_analysis),
                })),
                Err(e) => Ok(Json(ExplainResponse {
                    plan: Some(serde_json::json!({})),
                    error: Some(format!("Failed to serialize execution plan: {}", e)),
                    advisor_analysis: None,
                })),
            }
        }
        Err(e) => Ok(Json(ExplainResponse {
            plan: Some(serde_json::json!({})),
            error: Some(e.to_string()),
            advisor_analysis: None,
        })),
    }
}

/// Handle benchmark requests
async fn benchmark_handler(
    State(state): State<AppState>,
    Json(payload): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, StatusCode> {
    let config = payload.config.unwrap_or_default();
    let benchmark_suite =
        BenchmarkSuite::new(state.db.clone(), state.advisor.clone(), Some(config));

    match benchmark_suite.benchmark_query(&payload.query).await {
        Ok(result) => Ok(Json(BenchmarkResponse {
            result: Some(result),
            error: None,
        })),
        Err(e) => Ok(Json(BenchmarkResponse {
            result: None,
            error: Some(e.to_string()),
        })),
    }
}

/// Handle benchmark comparison requests
async fn benchmark_compare_handler(
    State(state): State<AppState>,
    Json(payload): Json<BenchmarkCompareRequest>,
) -> Result<Json<BenchmarkCompareResponse>, StatusCode> {
    let config = payload.config.unwrap_or_default();
    let benchmark_suite =
        BenchmarkSuite::new(state.db.clone(), state.advisor.clone(), Some(config));

    // Run benchmarks for both queries
    let result_a = benchmark_suite.benchmark_query(&payload.query_a).await;
    let result_b = benchmark_suite.benchmark_query(&payload.query_b).await;

    match (result_a, result_b) {
        (Ok(bench_a), Ok(bench_b)) => {
            let comparison = benchmark_suite.compare_benchmarks(
                &bench_a,
                &bench_b,
                payload.label_a,
                payload.label_b,
            );
            Ok(Json(BenchmarkCompareResponse {
                comparison: Some(comparison),
                error: None,
            }))
        }
        (Err(e), _) | (_, Err(e)) => Ok(Json(BenchmarkCompareResponse {
            comparison: None,
            error: Some(format!("Benchmark failed: {}", e)),
        })),
    }
}
