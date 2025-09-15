//! SQL Trace - A Web-Based SQL Query Visualizer & Advisor
//!
//! This tool helps developers understand and optimize SQL queries by visualizing
//! their execution plans in an interactive web interface.

#![warn(missing_docs)]

use clap::Parser;
use std::net::SocketAddr;
use tracing::{info, Level};

use sqltrace_rs::{
    advisor::QueryAdvisor,
    server::{create_router, AppState},
    Database,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Database connection string (e.g., postgres://user:password@localhost:5432/dbname)
    #[clap(short, long)]
    database_url: String,

    /// Port to run the web server on
    #[clap(short, long, default_value = "3000")]
    port: u16,

    /// Host to bind the web server to
    #[clap(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    setup_logging();

    // Parse command line arguments
    let args = Args::parse();

    // Initialize database connection
    let db = Database::new(&args.database_url).await?;
    info!("Connected to database");

    // Create application state
    let state = AppState {
        db,
        advisor: QueryAdvisor::new(),
    };

    // Build the router
    let app = create_router(state);

    // Create socket address
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    info!("Starting server on http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Configure logging for the application
fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();
}
