//! Data structures for PostgreSQL execution plans

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a single node in an execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNode {
    /// Type of the plan node (e.g., "Seq Scan", "Index Scan")
    #[serde(rename = "Node Type")]
    pub node_type: String,

    /// Name of the relation being accessed
    #[serde(rename = "Relation Name")]
    pub relation_name: Option<String>,

    /// Alias for the relation if one was used in the query
    #[serde(rename = "Alias")]
    pub alias: Option<String>,

    /// Estimated startup cost
    #[serde(rename = "Startup Cost")]
    pub startup_cost: f64,

    /// Estimated total cost
    #[serde(rename = "Total Cost")]
    pub total_cost: f64,

    /// Actual startup time in milliseconds
    #[serde(rename = "Actual Startup Time")]
    pub actual_startup_time: Option<f64>,

    /// Actual total time in milliseconds
    #[serde(rename = "Actual Total Time")]
    pub actual_total_time: f64,

    /// Actual number of rows returned by this node
    #[serde(rename = "Actual Rows")]
    pub actual_rows: u64,

    /// Number of loops executed by this node
    #[serde(rename = "Actual Loops")]
    pub actual_loops: u64,

    /// Child nodes in the execution plan
    #[serde(default, rename = "Plans")]
    pub plans: Vec<PlanNode>,

    /// Additional node-specific output
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl PlanNode {
    /// Get the actual time as a Duration
    ///
    /// Returns a `Duration` representing the time spent in this node.
    /// The duration is in milliseconds, matching PostgreSQL's EXPLAIN ANALYZE output.
    pub fn actual_duration(&self) -> Duration {
        Duration::from_millis((self.actual_total_time * self.actual_loops as f64) as u64)
    }
}

/// Represents a single plan in the PostgreSQL EXPLAIN output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlan {
    /// The execution plan
    #[serde(rename = "Plan")]
    pub plan: PlanNode,

    /// Planning time in milliseconds
    #[serde(rename = "Planning Time")]
    pub planning_time: f64,

    /// Execution time in milliseconds
    #[serde(rename = "Execution Time")]
    pub execution_time: f64,
}

/// Represents the top-level structure of a PostgreSQL EXPLAIN output
pub type ExplainOutput = Vec<ExplainPlan>;

/// Represents a complete execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// The root node of the execution plan
    pub root: PlanNode,

    /// Total planning time in milliseconds
    pub planning_time: f64,

    /// Total execution time in milliseconds
    pub execution_time: f64,
}

impl ExecutionPlan {
    /// Get the planning time as a Duration
    ///
    /// Converts the internal millisecond value to a `std::time::Duration`.
    pub fn planning_duration(&self) -> Duration {
        Duration::from_millis(self.planning_time as u64)
    }

    /// Get the execution time as a Duration
    ///
    /// Converts the internal millisecond value to a `std::time::Duration`.
    pub fn execution_duration(&self) -> Duration {
        Duration::from_millis(self.execution_time as u64)
    }
}
