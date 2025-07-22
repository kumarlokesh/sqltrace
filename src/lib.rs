//! SQLTrace - A Terminal-Based SQL Visualizer & Advisor

#![warn(missing_docs)]

pub mod db;
pub mod error;
pub mod ui;

/// Re-export common types for easier use in tests and examples
pub use db::Database;
pub use error::SqlTraceError;
