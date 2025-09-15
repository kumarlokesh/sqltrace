# SQLTrace

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/sqltrace-rs.svg)](https://crates.io/crates/sqltrace-rs)

A high-performance, web-based SQL query visualizer, performance analyzer, and optimization advisor that helps developers understand and optimize their database queries across multiple database engines.

## Features

### Query Analysis & Visualization

- **Interactive Web UI** for exploring query execution plans with mouse and keyboard
- **Multi-Engine Support** for PostgreSQL, MySQL, and SQLite databases
- **Plan Visualization** with expandable/collapsible nodes and detailed execution metrics
- **Performance Analytics** with real-time cost analysis and color-coded performance indicators
- **Interactive Examples** with one-click query execution for common patterns

### Performance & Benchmarking

- **Advanced Benchmarking Suite** with configurable warmup and benchmark runs
- **Statistical Analysis** including averages, percentiles, confidence intervals, and significance testing
- **Query Comparison** with side-by-side performance analysis and improvement percentages
- **Query History** with persistent storage and comparison capabilities

### Optimization & Advisory

- **Rule-based Query Advisor** with intelligent optimization suggestions
- **Performance Impact Scoring** with severity levels and specific recommendations
- **Plan Analysis** detecting sequential scans, missing indexes, and inefficient operations

### Export & Integration

- **Multiple Export Formats** supporting JSON, text, HTML, and clipboard copy
- **RESTful API** for programmatic access to all features
- **Lightweight & Fast** built with Rust backend for maximum performance

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Docker](https://www.docker.com/get-started) and [docker-compose](https://docs.docker.com/compose/install/) (for database setup)
- [just](https://github.com/casey/just) (optional, for running development tasks)

### Quick Start with Docker

The fastest way to get started is using our multi-engine Docker setup:

```bash
# Clone the repository
git clone https://github.com/kumarlokesh/sqltrace-rs.git
cd sqltrace-rs

# Start all database engines
docker-compose up -d postgres mysql sqlite

# Build and run SQLTrace
cargo run -- --database-url postgres://postgres:postgres@localhost:5432/sqltrace_dev
```

Then open <http://localhost:3000> in your browser!

## Testing

### Prerequisites

- [Docker](https://www.docker.com/get-started) (for running the test database)
- [docker-compose](https://docs.docker.com/compose/install/)

### Setting Up Test Database

Run the setup script to start a PostgreSQL container and set up the test database:

```bash
# Make the script executable
chmod +x ./scripts/setup_test_db.sh

# Run the setup script
./scripts/setup_test_db.sh
```

This will:

- Start a PostgreSQL container using Docker
- Wait for the database to be ready
- Create a test database
- Create a test environment file

When you're done testing, you can stop the container with:

```bash
docker-compose down
```

### Running Tests

#### Using just (recommended)

```bash
# Install just if you haven't
cargo install just

# Run tests
just test

# Run tests with coverage (requires cargo-tarpaulin)
just coverage

# Clean up test database
just clean-test-db
```

#### Using Cargo directly

```bash
# Load test environment
source tests/test.env

# Run tests
cargo test -- --test-threads=1
```

### Installation

```bash
# Install from crates.io (once published)
cargo install sqltrace-rs

# Or build from source
git clone https://github.com/kumarlokesh/sqltrace-rs.git
cd sqltrace-rs
cargo build --release
```

### Basic Usage

```bash
# PostgreSQL (recommended for full feature support)
sqltrace-rs --database-url postgres://postgres:postgres@localhost:5432/sqltrace_dev

# MySQL
sqltrace-rs --database-url mysql://mysql:mysql@localhost:3306/sqltrace_dev

# SQLite
sqltrace-rs --database-url sqlite:///path/to/database.db

# Custom host and port
sqltrace-rs --database-url postgres://user:password@localhost:5432/dbname --port 8080 --host 0.0.0.0

# Debug logging
RUST_LOG=debug sqltrace-rs --database-url postgres://user:password@localhost:5432/dbname
```

Then open your web browser and navigate to `http://localhost:3000` to access the interactive query visualizer.

### Web Interface

#### Query Analysis

- **Query Editor**: Write and edit SQL queries with syntax highlighting
- **Execute Button**: Analyze your query and generate execution plan visualization
- **Interactive Plan Tree**: Click to expand/collapse nodes in the execution plan
- **Detailed Metrics**: View cost, timing, and row information for each operation
- **Optimization Suggestions**: Get intelligent recommendations from the built-in advisor

#### Performance Benchmarking

- **Single Query Benchmarking**: Run configurable performance tests on individual queries
- **Query Comparison**: Compare performance between two different queries
- **Statistical Analysis**: View detailed performance statistics and confidence intervals
- **Results Export**: Save benchmark results in multiple formats

#### Database Management

- **Multi-Engine Support**: Switch between PostgreSQL, MySQL, and SQLite
- **Query History**: Access previously executed queries with persistent storage
- **Export Options**: Download plans and results in JSON, text, or HTML formats

## Database Setup

### Using Docker (Recommended)

```bash
# Start all database engines with sample data
docker-compose up -d

# Start specific engines only
docker-compose up -d postgres mysql

# Include database management UIs
docker-compose --profile admin up -d

# Access management interfaces:
# - pgAdmin: http://localhost:8080 (admin@sqltrace.dev / postgres)
# - phpMyAdmin: http://localhost:8081 (mysql / mysql)
```

### Manual Database Setup

#### PostgreSQL

```sql
CREATE DATABASE sqltrace_dev;
\c sqltrace_dev;
\i scripts/init_db.sql
\i scripts/sample_data.sql
```

#### MySQL

```sql
CREATE DATABASE sqltrace_dev;
USE sqltrace_dev;
source scripts/init_mysql.sql;
source scripts/sample_data_mysql.sql;
```

#### SQLite

```bash
sqlite3 sqltrace_dev.db < scripts/init_sqlite.sql
sqlite3 sqltrace_dev.db < scripts/sample_data_sqlite.sql
```

## Troubleshooting

### Connection Issues

```bash
# Test PostgreSQL connection
psql postgres://postgres:postgres@localhost:5432/sqltrace_dev -c "SELECT 1"

# Test MySQL connection
mysql -h localhost -u mysql -pmysql sqltrace_dev -e "SELECT 1"

# Test SQLite database
sqlite3 sqltrace_dev.db "SELECT 1"
```

### Permissions

For PostgreSQL, ensure the user has required permissions:

```sql
GRANT pg_read_all_stats TO your_username;
GRANT CONNECT ON DATABASE sqltrace_dev TO your_username;
GRANT USAGE ON SCHEMA public TO your_username;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO your_username;
```

### Performance Issues

```bash
# Check port availability
lsof -i :3000

# Use different port
sqltrace-rs --database-url postgres://... --port 8080

# Enable debug logging
RUST_LOG=debug sqltrace-rs --database-url postgres://...
```

## API Reference

### Query Analysis

```bash
# Analyze a query
curl -X POST http://localhost:3000/api/explain \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * FROM users WHERE age > 25"}'
```

### Benchmarking

```bash
# Benchmark a single query
curl -X POST http://localhost:3000/api/benchmark \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT * FROM users WHERE department = 'Engineering'",
    "warmup_runs": 3,
    "benchmark_runs": 10,
    "timeout_seconds": 30
  }'

# Compare two queries
curl -X POST http://localhost:3000/api/benchmark/compare \
  -H "Content-Type: application/json" \
  -d '{
    "query1": "SELECT * FROM users WHERE department = 'Engineering'",
    "query2": "SELECT * FROM users WHERE department = 'Marketing'",
    "warmup_runs": 3,
    "benchmark_runs": 10
  }'
```

## Architecture

```mermaid
graph TD
    A[Web Browser] --> B[HTTP Request]
    B --> C[Axum Web Server]
    C --> D[SQL Validation]
    D --> E[Database Connection]
    E --> F[EXPLAIN ANALYZE]
    F --> G[Plan Processing]
    G --> H[JSON Response]
    H --> I[Web UI Visualization]
    G --> J[Advisor Engine]
    J --> H
```

### Core Components

1. **Web Server**: Axum-based HTTP server serving the web interface and API
2. **Database Connector**: Handles connections to PostgreSQL databases
3. **Plan Parser**: Parses and normalizes execution plans from PostgreSQL
4. **REST API**: JSON endpoints for query execution and plan retrieval
5. **Web Frontend**: Modern JavaScript interface for interactive plan visualization
6. **Advisor Engine**: Analyzes plans and provides optimization suggestions

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.
