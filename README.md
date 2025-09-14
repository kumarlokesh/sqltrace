# SQLTrace

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/sqltrace-rs.svg)](https://crates.io/crates/sqltrace-rs)

A high-performance, web-based SQL query visualizer and advisor that helps developers understand and optimize their database queries.

## Features

- **Interactive Web UI** for exploring query execution plans with mouse and keyboard
- **PostgreSQL Support** with full EXPLAIN ANALYZE visualization
- **Plan Visualization** with expandable/collapsible nodes and detailed execution metrics
- **Performance Analytics** with real-time cost analysis and color-coded performance indicators
- **Interactive Examples** with one-click query execution for common patterns
- **Export Capabilities** supporting JSON, text formats, and clipboard copy
- **Dark/Light Themes** with persistent user preferences
- **Real-time Query Execution** via REST API endpoints
- **Lightweight & Fast** built with Rust backend for maximum performance

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [PostgreSQL](https://www.postgresql.org/download/) (for running tests)
- [just](https://github.com/casey/just) (optional, for running development tasks)

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

### Development Workflow

1. Make your changes
2. Run tests: `just test`
3. Run lints: `just lint`
4. Format code: `cargo fmt`

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
# Start the web server (default port 3000)
sqltrace-rs --database-url postgres://user:password@localhost:5432/dbname

# Use custom host and port
sqltrace-rs --database-url postgres://user:password@localhost:5432/dbname --port 8080 --host 0.0.0.0

# Run with debug logging (for troubleshooting)
RUST_LOG=debug sqltrace-rs --database-url postgres://user:password@localhost:5432/dbname
```

Then open your web browser and navigate to `http://localhost:3000` to access the interactive query visualizer.

### Web Interface

- **Query Editor**: Write and edit SQL queries with syntax highlighting
- **Execute Button**: Analyze your query and generate execution plan visualization
- **Interactive Plan Tree**: Click to expand/collapse nodes in the execution plan
- **Detailed Metrics**: View cost, timing, and row information for each operation
- **Error Handling**: Clear error messages for invalid queries or connection issues

## Troubleshooting

### Common Issues

- **Connection Issues**: Ensure your database is running and accessible

  ```bash
  # Test database connection
  psql postgres://user:password@localhost:5432/dbname -c "SELECT 1"
  ```

- **Permissions**: The database user needs permission to run EXPLAIN ANALYZE

  ```sql
  GRANT pg_read_all_stats TO your_username;
  ```

- **Web Server Issues**: Check if the port is already in use

  ```bash
  # Check what's using port 3000
  lsof -i :3000
  
  # Use a different port if needed
  sqltrace-rs --database-url postgres://... --port 8080
  ```

- **Debugging**: Enable debug logging for more information

  ```bash
  RUST_LOG=debug sqltrace-rs --database-url postgres://...
  ```

## Recent Changes

### 0.1.0 (Unreleased)

- Modern, responsive web UI with interactive execution plan visualization
- RESTful API for programmatic access to query analysis
- Improved error handling and user feedback
- Enhanced plan visualization with expandable/collapsible nodes
- Real-time query execution and plan generation

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

## Roadmap

### Phase 1: MVP (PostgreSQL)

- [x] Project setup and basic structure
- [x] PostgreSQL connection and query execution
  - [x] Database connection pooling
  - [x] Query execution with EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
  - [x] Execution plan parsing and deserialization
  - [x] Comprehensive test coverage for various query types
- [x] Web-based UI for plan visualization  
- [x] REST API endpoints for query execution
- [x] Interactive plan tree with expand/collapse functionality

### Phase 2: Enhanced Features

- [x] Performance metrics and analysis
  - [x] Real-time cost analysis with color-coded indicators
  - [x] Execution time and row count metrics
  - [x] Plan node statistics dashboard
- [x] Export functionality
  - [x] JSON format export
  - [x] Text format export  
  - [x] Copy to clipboard with visual feedback
- [x] Interactive example queries
- [x] Dark/light theme toggle with persistent preferences
- [ ] Rule-based optimization suggestions
- [ ] Query history and comparison

### Phase 3: Multi-engine Support

- [ ] MySQL support
- [ ] SQLite support
- [ ] Abstract plan parser interface

### Phase 4: Advanced Features

- [x] Export plans (JSON, Text formats)
- [ ] Export plans (HTML, PNG formats)
- [ ] Plugin system for custom visualizations
- [ ] Benchmarking tools
- [ ] Rule-based optimization advisor engine

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.
