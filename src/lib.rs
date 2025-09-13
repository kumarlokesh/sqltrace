//! SQLTrace Library
//!
//! A high-performance web-based SQL query visualizer and advisor for PostgreSQL.
//!
//! This library provides functionality for analyzing and visualizing SQL query execution plans
//! in a web-based user interface. It's designed to help developers understand and optimize
//! their database queries by providing detailed insights into query execution.
//!
//! ## Features
//! - Execution plan visualization via web interface
//! - Query analysis and optimization suggestions  
//! - Support for PostgreSQL (with plans to add more database backends)
//! - REST API for programmatic access

#![warn(missing_docs)]

/// Database connection and query execution functionality.
pub mod db;

/// Error types and result handling for the SQLTrace application.
pub mod error;

/// Web server setup and configuration.
pub mod server;

/// UI utilities and data structures for rendering execution plans.
pub mod ui;

/// Web-related utilities and validation functions.
pub mod web;

/// Re-export common types for easier use in tests and examples
pub use db::Database;
pub use error::SqlTraceError;
pub use server::{create_router, AppState};
