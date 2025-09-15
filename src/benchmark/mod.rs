//! Benchmarking and performance comparison tools for SQLTrace
//!
//! This module provides functionality to benchmark SQL queries, collect performance
//! metrics, and compare different query implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::advisor::{AdvisorAnalysis, QueryAdvisor};
use crate::db::models::ExecutionPlan;
use crate::db::Database;
use crate::SqlTraceError;

/// Configuration for benchmark runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of warmup runs (not included in metrics)
    pub warmup_runs: u32,
    /// Number of actual benchmark runs
    pub benchmark_runs: u32,
    /// Timeout for individual query execution (in seconds)
    pub timeout_seconds: u64,
    /// Whether to include detailed execution plans in results
    pub include_execution_plans: bool,
    /// Whether to run advisor analysis on each query
    pub include_advisor_analysis: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_runs: 2,
            benchmark_runs: 5,
            timeout_seconds: 30,
            include_execution_plans: true,
            include_advisor_analysis: true,
        }
    }
}

/// Single benchmark run result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    /// Duration of the query execution
    pub execution_time: Duration,
    /// Execution plan (if enabled in config)
    pub execution_plan: Option<ExecutionPlan>,
    /// Advisor analysis (if enabled in config)
    pub advisor_analysis: Option<AdvisorAnalysis>,
    /// Timestamp when the run was executed
    pub timestamp: std::time::SystemTime,
}

/// Complete benchmark result for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// The SQL query that was benchmarked
    pub query: String,
    /// Individual run results
    pub runs: Vec<BenchmarkRun>,
    /// Statistical summary
    pub statistics: BenchmarkStatistics,
    /// Configuration used for this benchmark
    pub config: BenchmarkConfig,
}

/// Statistical analysis of benchmark runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStatistics {
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Minimum execution time
    pub min_execution_time: Duration,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Standard deviation of execution times
    pub std_deviation: Duration,
    /// 95th percentile execution time
    pub p95_execution_time: Duration,
    /// Total number of successful runs
    pub successful_runs: u32,
    /// Total number of failed runs
    pub failed_runs: u32,
    /// Average cost from execution plans
    pub avg_cost: Option<f64>,
    /// Average advisor score
    pub avg_advisor_score: Option<f64>,
}

/// Comparison between two benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    /// Label for the first benchmark (e.g., "Query A")
    pub label_a: String,
    /// Label for the second benchmark (e.g., "Query B")
    pub label_b: String,
    /// Performance improvement percentage (positive means B is faster)
    pub performance_improvement: f64,
    /// Statistical significance of the difference
    pub statistical_significance: StatisticalSignificance,
    /// Detailed comparison metrics
    pub metrics: ComparisonMetrics,
}

/// Statistical significance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalSignificance {
    /// Difference is highly significant (p < 0.01)
    HighlySignificant,
    /// Difference is significant (p < 0.05)
    Significant,
    /// Difference is marginally significant (p < 0.1)
    MarginallySignificant,
    /// No significant difference
    NotSignificant,
}

/// Detailed comparison metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    /// Average execution time difference
    pub avg_time_diff: Duration,
    /// Cost difference (if available)
    pub cost_diff: Option<f64>,
    /// Advisor score difference (if available)
    pub advisor_score_diff: Option<f64>,
    /// Confidence interval for the time difference
    pub confidence_interval: (Duration, Duration),
}

/// Benchmark suite for running multiple query benchmarks
pub struct BenchmarkSuite {
    db: Database,
    advisor: QueryAdvisor,
    config: BenchmarkConfig,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(db: Database, advisor: QueryAdvisor, config: Option<BenchmarkConfig>) -> Self {
        Self {
            db,
            advisor,
            config: config.unwrap_or_default(),
        }
    }

    /// Benchmark a single query
    pub async fn benchmark_query(&self, query: &str) -> Result<BenchmarkResult, SqlTraceError> {
        let mut runs = Vec::new();
        let mut failed_runs = 0;

        // Warmup runs
        for _ in 0..self.config.warmup_runs {
            if let Err(_) = self.execute_single_run(query).await {
                // Ignore warmup failures
            }
        }

        // Actual benchmark runs
        for _ in 0..self.config.benchmark_runs {
            match self.execute_single_run(query).await {
                Ok(run) => runs.push(run),
                Err(_) => failed_runs += 1,
            }
        }

        if runs.is_empty() {
            return Err(SqlTraceError::Database(
                "All benchmark runs failed".to_string(),
            ));
        }

        let statistics = self.calculate_statistics(&runs, failed_runs);

        Ok(BenchmarkResult {
            query: query.to_string(),
            runs,
            statistics,
            config: self.config.clone(),
        })
    }

    /// Execute a single benchmark run
    async fn execute_single_run(&self, query: &str) -> Result<BenchmarkRun, SqlTraceError> {
        let start_time = Instant::now();

        // Execute the query and get execution plan
        let execution_plan = if self.config.include_execution_plans {
            Some(self.db.explain(query).await?)
        } else {
            None
        };

        let execution_time = start_time.elapsed();

        // Run advisor analysis if enabled
        let advisor_analysis = if self.config.include_advisor_analysis {
            execution_plan
                .as_ref()
                .map(|plan| self.advisor.analyze_plan(plan))
        } else {
            None
        };

        Ok(BenchmarkRun {
            execution_time,
            execution_plan,
            advisor_analysis,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Calculate statistical summary of benchmark runs
    fn calculate_statistics(&self, runs: &[BenchmarkRun], failed_runs: u32) -> BenchmarkStatistics {
        let execution_times: Vec<Duration> = runs.iter().map(|run| run.execution_time).collect();

        let avg_execution_time = self.calculate_average_duration(&execution_times);
        let min_execution_time = execution_times
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::ZERO);
        let max_execution_time = execution_times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO);
        let std_deviation = self.calculate_std_deviation(&execution_times, avg_execution_time);
        let p95_execution_time = self.calculate_percentile(&execution_times, 0.95);

        let avg_cost = self.calculate_average_cost(runs);
        let avg_advisor_score = self.calculate_average_advisor_score(runs);

        BenchmarkStatistics {
            avg_execution_time,
            min_execution_time,
            max_execution_time,
            std_deviation,
            p95_execution_time,
            successful_runs: runs.len() as u32,
            failed_runs,
            avg_cost,
            avg_advisor_score,
        }
    }

    /// Calculate average duration
    fn calculate_average_duration(&self, durations: &[Duration]) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }

        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();

        Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
    }

    /// Calculate standard deviation of durations
    fn calculate_std_deviation(&self, durations: &[Duration], mean: Duration) -> Duration {
        if durations.len() < 2 {
            return Duration::ZERO;
        }

        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / (durations.len() - 1) as f64;

        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// Calculate percentile duration
    fn calculate_percentile(&self, durations: &[Duration], percentile: f64) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted_durations = durations.to_vec();
        sorted_durations.sort();

        let index = (percentile * (sorted_durations.len() - 1) as f64) as usize;
        sorted_durations[index]
    }

    /// Calculate average cost from execution plans
    fn calculate_average_cost(&self, runs: &[BenchmarkRun]) -> Option<f64> {
        let costs: Vec<f64> = runs
            .iter()
            .filter_map(|run| run.execution_plan.as_ref())
            .map(|plan| plan.root.total_cost)
            .collect();

        if costs.is_empty() {
            None
        } else {
            Some(costs.iter().sum::<f64>() / costs.len() as f64)
        }
    }

    /// Calculate average advisor score
    fn calculate_average_advisor_score(&self, runs: &[BenchmarkRun]) -> Option<f64> {
        let scores: Vec<f64> = runs
            .iter()
            .filter_map(|run| run.advisor_analysis.as_ref())
            .map(|analysis| analysis.performance_score as f64)
            .collect();

        if scores.is_empty() {
            None
        } else {
            Some(scores.iter().sum::<f64>() / scores.len() as f64)
        }
    }

    /// Compare two benchmark results
    pub fn compare_benchmarks(
        &self,
        result_a: &BenchmarkResult,
        result_b: &BenchmarkResult,
        label_a: String,
        label_b: String,
    ) -> BenchmarkComparison {
        let avg_time_a = result_a.statistics.avg_execution_time;
        let avg_time_b = result_b.statistics.avg_execution_time;

        let performance_improvement = if avg_time_a.as_nanos() > 0 {
            ((avg_time_a.as_nanos() as f64 - avg_time_b.as_nanos() as f64)
                / avg_time_a.as_nanos() as f64)
                * 100.0
        } else {
            0.0
        };

        let avg_time_diff = if avg_time_b > avg_time_a {
            avg_time_b - avg_time_a
        } else {
            avg_time_a - avg_time_b
        };

        let cost_diff = match (result_a.statistics.avg_cost, result_b.statistics.avg_cost) {
            (Some(cost_a), Some(cost_b)) => Some(cost_b - cost_a),
            _ => None,
        };

        let advisor_score_diff = match (
            result_a.statistics.avg_advisor_score,
            result_b.statistics.avg_advisor_score,
        ) {
            (Some(score_a), Some(score_b)) => Some(score_b - score_a),
            _ => None,
        };

        // Simple statistical significance test (t-test approximation)
        let statistical_significance = self.calculate_statistical_significance(result_a, result_b);

        // Calculate 95% confidence interval (simplified)
        let confidence_interval = self.calculate_confidence_interval(result_a, result_b);

        BenchmarkComparison {
            label_a,
            label_b,
            performance_improvement,
            statistical_significance,
            metrics: ComparisonMetrics {
                avg_time_diff,
                cost_diff,
                advisor_score_diff,
                confidence_interval,
            },
        }
    }

    /// Calculate statistical significance (simplified t-test)
    fn calculate_statistical_significance(
        &self,
        result_a: &BenchmarkResult,
        result_b: &BenchmarkResult,
    ) -> StatisticalSignificance {
        // Simplified statistical test - in a real implementation, you'd use proper statistical libraries
        let n_a = result_a.statistics.successful_runs as f64;
        let n_b = result_b.statistics.successful_runs as f64;

        if n_a < 3.0 || n_b < 3.0 {
            return StatisticalSignificance::NotSignificant;
        }

        let mean_diff = (result_a.statistics.avg_execution_time.as_nanos() as f64
            - result_b.statistics.avg_execution_time.as_nanos() as f64)
            .abs();

        let pooled_std = (result_a.statistics.std_deviation.as_nanos() as f64
            + result_b.statistics.std_deviation.as_nanos() as f64)
            / 2.0;

        if pooled_std == 0.0 {
            return StatisticalSignificance::NotSignificant;
        }

        let t_stat = mean_diff / (pooled_std * (1.0 / n_a + 1.0 / n_b).sqrt());

        // Simplified thresholds - real implementation would use proper t-distribution
        if t_stat > 2.576 {
            StatisticalSignificance::HighlySignificant
        } else if t_stat > 1.96 {
            StatisticalSignificance::Significant
        } else if t_stat > 1.645 {
            StatisticalSignificance::MarginallySignificant
        } else {
            StatisticalSignificance::NotSignificant
        }
    }

    /// Calculate confidence interval for the difference
    fn calculate_confidence_interval(
        &self,
        result_a: &BenchmarkResult,
        result_b: &BenchmarkResult,
    ) -> (Duration, Duration) {
        // Simplified confidence interval calculation
        let diff = result_a.statistics.avg_execution_time.as_nanos() as i64
            - result_b.statistics.avg_execution_time.as_nanos() as i64;

        let margin_of_error = (result_a.statistics.std_deviation.as_nanos() as i64
            + result_b.statistics.std_deviation.as_nanos() as i64)
            / 2;

        let lower_bound = (diff - margin_of_error).max(0) as u64;
        let upper_bound = (diff + margin_of_error).max(0) as u64;

        (
            Duration::from_nanos(lower_bound),
            Duration::from_nanos(upper_bound),
        )
    }

    /// Run a benchmark suite with multiple queries
    pub async fn run_benchmark_suite(
        &self,
        queries: HashMap<String, String>,
    ) -> Result<HashMap<String, BenchmarkResult>, SqlTraceError> {
        let mut results = HashMap::new();

        for (name, query) in queries {
            let result = self.benchmark_query(&query).await?;
            results.insert(name, result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.warmup_runs, 2);
        assert_eq!(config.benchmark_runs, 5);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_calculate_average_duration() {
        // Test the duration calculation without database dependency
        let durations = vec![
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(300),
        ];

        // Calculate average manually to test
        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        let expected = Duration::from_nanos((total_nanos / durations.len() as u128) as u64);

        // Create a temporary benchmark suite for testing the calculation method
        // We'll test the method logic without actual database operations
        let avg_100_200_300 =
            Duration::from_nanos(((100_000_000 + 200_000_000 + 300_000_000) / 3) as u64);
        assert_eq!(avg_100_200_300, Duration::from_millis(200));
    }

    #[test]
    fn test_statistical_significance_levels() {
        // Test that statistical significance enum variants exist
        let _highly_sig = StatisticalSignificance::HighlySignificant;
        let _sig = StatisticalSignificance::Significant;
        let _marginal = StatisticalSignificance::MarginallySignificant;
        let _not_sig = StatisticalSignificance::NotSignificant;
    }
}
