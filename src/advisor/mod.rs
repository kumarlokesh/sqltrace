//! Query optimization advisor module
//!
//! This module provides rule-based analysis of PostgreSQL execution plans
//! and suggests optimizations to improve query performance.

use crate::db::models::{ExecutionPlan, PlanNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Type of suggestion (e.g., "Index", "Query Rewrite", "Schema")
    pub suggestion_type: String,
    /// Severity level (High, Medium, Low)
    pub severity: Severity,
    /// Human-readable title
    pub title: String,
    /// Detailed description of the issue
    pub description: String,
    /// Suggested action to take
    pub recommendation: String,
    /// Node index this suggestion applies to (if specific)
    pub node_index: Option<usize>,
    /// Estimated impact if implemented
    pub impact: String,
}

/// Severity level of optimization suggestions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// High priority issues that significantly impact performance
    High,
    /// Medium priority issues with moderate performance impact
    Medium,
    /// Low priority issues or minor optimizations
    Low,
}

/// Complete advisor analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorAnalysis {
    /// List of optimization suggestions
    pub suggestions: Vec<OptimizationSuggestion>,
    /// Overall performance score (0-100)
    pub performance_score: u8,
    /// Summary statistics
    pub summary: AnalysisSummary,
}

/// Analysis summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Total number of suggestions
    pub total_suggestions: usize,
    /// Number of high severity issues
    pub high_severity_count: usize,
    /// Most expensive operation type
    pub most_expensive_operation: String,
    /// Total estimated cost
    pub total_cost: f64,
    /// Potential improvement estimate
    pub potential_improvement: String,
}

/// Query optimization advisor
#[derive(Debug, Clone)]
pub struct QueryAdvisor {
    /// Rule configurations
    config: AdvisorConfig,
}

/// Configuration for the advisor engine
#[derive(Debug, Clone)]
pub struct AdvisorConfig {
    /// Cost threshold for expensive operations
    pub expensive_cost_threshold: f64,
    /// Row count threshold for large scans
    pub large_scan_threshold: u64,
    /// Enable index suggestions
    pub enable_index_suggestions: bool,
    /// Enable query rewrite suggestions
    pub enable_rewrite_suggestions: bool,
}

impl Default for AdvisorConfig {
    fn default() -> Self {
        Self {
            expensive_cost_threshold: 1000.0,
            large_scan_threshold: 10000,
            enable_index_suggestions: true,
            enable_rewrite_suggestions: true,
        }
    }
}

impl QueryAdvisor {
    /// Create a new query advisor with default configuration
    pub fn new() -> Self {
        Self {
            config: AdvisorConfig::default(),
        }
    }

    /// Create a new query advisor with custom configuration
    pub fn with_config(config: AdvisorConfig) -> Self {
        Self { config }
    }

    /// Analyze an execution plan and provide optimization suggestions
    pub fn analyze_plan(&self, plan: &ExecutionPlan) -> AdvisorAnalysis {
        let mut suggestions = Vec::new();
        let mut node_costs = HashMap::new();

        self.analyze_node(&plan.root, &mut suggestions, &mut node_costs, 0);

        let summary = self.generate_summary(&suggestions, &node_costs, plan);
        let performance_score = self.calculate_performance_score(&suggestions, plan);

        AdvisorAnalysis {
            suggestions,
            performance_score,
            summary,
        }
    }

    /// Recursively analyze plan nodes
    fn analyze_node(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_costs: &mut HashMap<String, f64>,
        node_index: usize,
    ) {
        node_costs.insert(node.node_type.clone(), node.total_cost);

        // Apply optimization rules
        self.check_sequential_scan(node, suggestions, node_index);
        self.check_expensive_operations(node, suggestions, node_index);
        self.check_nested_loops(node, suggestions, node_index);
        self.check_large_sorts(node, suggestions, node_index);
        self.check_missing_indexes(node, suggestions, node_index);
        self.check_inefficient_joins(node, suggestions, node_index);

        for (i, child) in node.plans.iter().enumerate() {
            self.analyze_node(child, suggestions, node_costs, node_index + i + 1);
        }
    }

    /// Check for expensive sequential scans
    fn check_sequential_scan(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_index: usize,
    ) {
        if node.node_type == "Seq Scan" && node.total_cost > self.config.expensive_cost_threshold {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "Index".to_string(),
                severity: Severity::High,
                title: "Expensive Sequential Scan Detected".to_string(),
                description: format!(
                    "Sequential scan on table '{}' has high cost ({:.2}). This indicates the entire table is being scanned.",
                    node.relation_name.as_deref().unwrap_or("unknown"),
                    node.total_cost
                ),
                recommendation: "Consider adding an index on frequently queried columns or adding WHERE clauses to reduce rows scanned.".to_string(),
                node_index: Some(node_index),
                impact: "High - Could significantly reduce query execution time".to_string(),
            });
        }
    }

    /// Check for expensive operations in general
    fn check_expensive_operations(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_index: usize,
    ) {
        if node.total_cost > self.config.expensive_cost_threshold * 2.0 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "Performance".to_string(),
                severity: Severity::Medium,
                title: format!("Expensive {} Operation", node.node_type),
                description: format!(
                    "{} operation has very high cost ({:.2}). This is significantly above average.",
                    node.node_type, node.total_cost
                ),
                recommendation: "Review query logic, consider query rewriting, or check if statistics are up to date.".to_string(),
                node_index: Some(node_index),
                impact: "Medium - May benefit from optimization".to_string(),
            });
        }
    }

    /// Check for inefficient nested loop joins
    fn check_nested_loops(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_index: usize,
    ) {
        if node.node_type == "Nested Loop" && node.actual_rows > self.config.large_scan_threshold {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "Join".to_string(),
                severity: Severity::High,
                title: "Inefficient Nested Loop Join".to_string(),
                description: format!(
                    "Nested loop join processing {} rows. This join method is inefficient for large datasets.",
                    node.actual_rows
                ),
                recommendation: "Consider adding indexes on join columns or restructuring the query to use hash or merge joins.".to_string(),
                node_index: Some(node_index),
                impact: "High - Could dramatically improve join performance".to_string(),
            });
        }
    }

    /// Check for large sort operations
    fn check_large_sorts(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_index: usize,
    ) {
        if node.node_type == "Sort" && node.actual_rows > self.config.large_scan_threshold {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "Index".to_string(),
                severity: Severity::Medium,
                title: "Large Sort Operation".to_string(),
                description: format!(
                    "Sort operation processing {} rows. Large sorts can be memory intensive.",
                    node.actual_rows
                ),
                recommendation: "Consider adding an index on the ORDER BY columns to avoid sorting, or limit result sets.".to_string(),
                node_index: Some(node_index),
                impact: "Medium - Could reduce memory usage and improve performance".to_string(),
            });
        }
    }

    /// Check for missing indexes (heuristic-based)
    fn check_missing_indexes(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_index: usize,
    ) {
        if !self.config.enable_index_suggestions {
            return;
        }

        // Check for filter conditions that might benefit from indexes
        if let Some(extra) = node.extra.as_object() {
            if let Some(filter) = extra.get("Filter") {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: "Index".to_string(),
                    severity: Severity::Medium,
                    title: "Potential Index Opportunity".to_string(),
                    description: format!(
                        "Filter condition detected: {}. This might benefit from an index.",
                        filter.as_str().unwrap_or("complex condition")
                    ),
                    recommendation: "Consider creating an index on the filtered column(s) to improve query performance.".to_string(),
                    node_index: Some(node_index),
                    impact: "Medium - Could improve filtering performance".to_string(),
                });
            }
        }
    }

    /// Check for inefficient join strategies
    fn check_inefficient_joins(
        &self,
        node: &PlanNode,
        suggestions: &mut Vec<OptimizationSuggestion>,
        node_index: usize,
    ) {
        if node.node_type.contains("Join") && node.total_cost > self.config.expensive_cost_threshold
        {
            let join_type = &node.node_type;
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "Join".to_string(),
                severity: Severity::Medium,
                title: format!("Expensive {} Operation", join_type),
                description: format!(
                    "{} has high cost ({:.2}). The join strategy may not be optimal.",
                    join_type, node.total_cost
                ),
                recommendation: "Consider adding indexes on join columns, updating table statistics, or restructuring the query.".to_string(),
                node_index: Some(node_index),
                impact: "Medium to High - Join optimization can significantly improve performance".to_string(),
            });
        }
    }

    /// Generate analysis summary
    fn generate_summary(
        &self,
        suggestions: &[OptimizationSuggestion],
        node_costs: &HashMap<String, f64>,
        plan: &ExecutionPlan,
    ) -> AnalysisSummary {
        let high_severity_count = suggestions
            .iter()
            .filter(|s| matches!(s.severity, Severity::High))
            .count();

        let most_expensive_operation = node_costs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(op, _)| op.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let potential_improvement = match high_severity_count {
            0 => "Low - Query appears well optimized".to_string(),
            1..=2 => "Medium - Some optimization opportunities available".to_string(),
            _ => "High - Significant optimization potential".to_string(),
        };

        AnalysisSummary {
            total_suggestions: suggestions.len(),
            high_severity_count,
            most_expensive_operation,
            total_cost: plan.root.total_cost,
            potential_improvement,
        }
    }

    /// Calculate overall performance score (0-100)
    fn calculate_performance_score(
        &self,
        suggestions: &[OptimizationSuggestion],
        plan: &ExecutionPlan,
    ) -> u8 {
        let mut score = 100u8;

        // Deduct points for suggestions
        for suggestion in suggestions {
            let deduction = match suggestion.severity {
                Severity::High => 20,
                Severity::Medium => 10,
                Severity::Low => 5,
            };
            score = score.saturating_sub(deduction);
        }

        // Consider execution time if available
        if plan.execution_time > 1000.0 {
            score = score.saturating_sub(10);
        }

        // Ensure minimum score
        score.max(10)
    }
}

impl Default for QueryAdvisor {
    fn default() -> Self {
        Self::new()
    }
}
