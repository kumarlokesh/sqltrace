//! # SQLTrace - A Terminal-Based SQL Visualizer & Advisor
//!
//! This library provides functionality for analyzing and visualizing SQL query execution plans
//! in a terminal-based user interface. It's designed to help developers understand and optimize
//! their database queries by providing detailed insights into query execution.
//!
//! ## Features
//! - Execution plan visualization
// - Query analysis and optimization suggestions
//! - Support for PostgreSQL (with plans to add more database backends)
//! - Interactive terminal-based user interface

#![warn(missing_docs)]

/// Database connection and query execution functionality.
pub mod db;

/// Error types and result handling for the SQLTrace application.
pub mod error;

/// Terminal user interface components for displaying and interacting with execution plans.
pub mod ui;

/// Re-export common types for easier use in tests and examples
pub use db::Database;
pub use error::SqlTraceError;
