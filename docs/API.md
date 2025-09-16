# API Reference

SQLTrace provides a RESTful API for programmatic access to all features.

## Query Analysis

### Analyze Query

Analyze a SQL query and get execution plan with optimization suggestions.

```bash
curl -X POST http://localhost:3000/api/explain \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * FROM users WHERE age > 25"}'
```

**Response:**
```json
{
  "plan": {
    "nodes": [...],
    "root_indices": [0]
  },
  "error": null,
  "advisor_analysis": {
    "suggestions": [...],
    "performance_score": 85,
    "summary": {...}
  }
}
```

## Benchmarking

### Single Query Benchmark

Run performance benchmarks on a single query.

```bash
curl -X POST http://localhost:3000/api/benchmark \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT * FROM users WHERE department = '\''Engineering'\''",
    "warmup_runs": 3,
    "benchmark_runs": 10,
    "timeout_seconds": 30
  }'
```

### Compare Queries

Compare performance between two different queries.

```bash
curl -X POST http://localhost:3000/api/benchmark/compare \
  -H "Content-Type: application/json" \
  -d '{
    "query1": "SELECT * FROM users WHERE department = '\''Engineering'\''",
    "query2": "SELECT * FROM users WHERE department = '\''Marketing'\''",
    "warmup_runs": 3,
    "benchmark_runs": 10
  }'
```

**Response:**
```json
{
  "result_a": {...},
  "result_b": {...},
  "comparison": {
    "performance_improvement_percent": 15.2,
    "statistical_significance": "Significant",
    "confidence_interval": {...}
  }
}
```

## Health Check

Check if the service is running and database is accessible.

```bash
curl http://localhost:3000/api/health
```

**Response:**
```json
{
  "status": "healthy",
  "database": "connected"
}
```

## Request/Response Formats

### Common Request Parameters

- `query` (string): SQL query to analyze or benchmark
- `warmup_runs` (integer, optional): Number of warmup runs before benchmarking (default: 3)
- `benchmark_runs` (integer, optional): Number of benchmark iterations (default: 10)
- `timeout_seconds` (integer, optional): Query timeout in seconds (default: 30)

### Error Responses

All endpoints return errors in this format:

```json
{
  "error": "Description of the error",
  "plan": null,
  "advisor_analysis": null
}
```

Common HTTP status codes:
- `200 OK`: Request successful (even with query errors)
- `400 Bad Request`: Invalid request format
- `500 Internal Server Error`: Server error
