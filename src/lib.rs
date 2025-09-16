//! SQLTrace Library
//!
//! A high-performance web-based SQL query visualizer and advisor for PostgreSQL, MySQL, and SQLite.
//!
//! This crate provides tools for analyzing and visualizing query execution plans.
//! It includes a web-based interface for interactive plan exploration and performance analysis.
//!
//! # Features
//!
//! - PostgreSQL execution plan parsing and analysis
//! - MySQL execution plan parsing and analysis
//! - SQLite execution plan parsing and analysis
//! - Interactive web-based visualization
//! - REST API for programmatic access
//! - Performance metrics and optimization insights
//! - Rule-based optimization advisor
//!
//! # Example
//!
//! ```no_run
//! use sqltrace_rs::{Database, server::create_router};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let db = Database::new("postgres://user:pass@localhost/db").await?;
//!     let app = create_router(db).await;
//!     // Start server...
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub mod advisor;
pub mod benchmark;
pub mod db;
pub mod error;
pub mod server;
pub mod ui;
pub mod web;

/// Re-export common types for easier use in tests and examples
pub use db::Database;
pub use error::SqlTraceError;
pub use server::{create_router, AppState};
