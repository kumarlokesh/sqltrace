//! UI utilities module for SQL Trace
//!
//! This module contains shared UI utilities and data structures for rendering execution plans.

use crate::db::models::{ExecutionPlan, PlanNode};
use serde::{Deserialize, Serialize};

/// Tree structure for representing execution plans in a hierarchical format
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PlanTree {
    /// All nodes in the tree
    pub nodes: Vec<PlanNodeUI>,
    /// Indices of root nodes
    pub root_indices: Vec<usize>,
    /// Hash of the last processed plan for caching
    pub last_plan_hash: Option<u64>,
}

/// UI representation of a plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNodeUI {
    /// Whether this node is expanded in the tree view
    pub expanded: bool,
    /// Indices of child nodes
    pub children: Vec<usize>,
    /// Node type (e.g., "Seq Scan", "Index Scan")
    pub node_type: String,
    /// Relation name if applicable
    pub relation_name: Option<String>,
    /// Alias if used
    pub alias: Option<String>,
    /// Estimated startup cost
    pub startup_cost: f64,
    /// Estimated total cost
    pub total_cost: f64,
    /// Actual startup time in milliseconds
    pub actual_startup_time: Option<f64>,
    /// Actual total time in milliseconds
    pub actual_total_time: f64,
    /// Actual number of rows returned
    pub actual_rows: u64,
    /// Additional node information
    pub extra: serde_json::Value,
}

/// Convert an execution plan into a tree structure suitable for web UI
pub fn build_plan_tree_ui(
    node: &PlanNode,
    tree: &mut PlanTree,
    level: usize,
    parent_idx: Option<usize>,
) -> usize {
    let node_idx = tree.nodes.len();

    // Create UI node with mapped fields
    let ui_node = PlanNodeUI {
        expanded: level < 2, // Auto-expand first two levels
        children: Vec::new(),
        node_type: node.node_type.clone(),
        relation_name: node.relation_name.clone(),
        alias: node.alias.clone(),
        startup_cost: node.startup_cost,
        total_cost: node.total_cost,
        actual_startup_time: node.actual_startup_time,
        actual_total_time: node.actual_total_time,
        actual_rows: node.actual_rows,
        extra: node.extra.clone(),
    };

    tree.nodes.push(ui_node);

    // If this is a root node, add to root indices
    if parent_idx.is_none() {
        tree.root_indices.push(node_idx);
    }

    // Process children
    if !node.plans.is_empty() {
        let mut child_indices = Vec::new();
        for child in &node.plans {
            let child_idx = build_plan_tree_ui(child, tree, level + 1, Some(node_idx));
            child_indices.push(child_idx);
        }
        tree.nodes[node_idx].children = child_indices;

        // Update parent's children if it exists
        if let Some(parent_idx) = parent_idx {
            tree.nodes[parent_idx].children.push(node_idx);
        }
    }

    node_idx
}

/// Convert execution plan to a format suitable for web frontend
pub fn plan_to_web_format(plan: &ExecutionPlan) -> serde_json::Value {
    let mut tree = PlanTree::default();
    build_plan_tree_ui(&plan.root, &mut tree, 0, None);

    serde_json::to_value(tree).unwrap_or_else(|_| serde_json::json!({}))
}
