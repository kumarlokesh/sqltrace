# SQLTrace

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/kumarlokesh/sqltrace-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/kumarlokesh/sqltrace-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/sqltrace-rs.svg)](https://crates.io/crates/sqltrace-rs)

A high-performance, terminal-based SQL query visualizer and advisor that helps developers understand and optimize their database queries.

## ✨ Features

- **Interactive TUI** for exploring query execution plans
- **Multi-database support** (PostgreSQL first, with more to come)
- **Detailed visualizations** of query execution paths
- **Performance insights** and optimization suggestions
- **Lightweight** and fast, built with Rust for maximum performance

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [PostgreSQL](https://www.postgresql.org/download/) (for running tests)
- [just](https://github.com/casey/just) (optional, for running development tasks)

## 🧪 Testing

### Prerequisites

- [Docker](https://www.docker.com/get-started) (for running the test database)
- [docker-compose](https://docs.docker.com/compose/install/) (usually comes with Docker Desktop)

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

## 🛠 Development

- Rust (latest stable version)
- PostgreSQL (for the initial version)

### Installation

```bash
# Install from crates.io (once published)
cargo install sqltrace-rs

# Or build from source
git clone https://github.com/kumarlokesh/sqltrace-rs.git
cd sqltrace-rs
cargo install --path .
```

### Basic Usage

```bash
# Connect to a PostgreSQL database and start the interactive TUI
sqltrace postgres://user:password@localhost:5432/dbname

# Or analyze a specific query
sqltrace postgres://user:password@localhost:5432/dbname --query "SELECT * FROM users WHERE id = 1"
```

## 🏗️ Architecture

```mermaid
graph TD
    A[User Input] --> B[SQL Query]
    B --> C[Database Connection]
    C --> D[EXPLAIN ANALYZE]
    D --> E[Plan Processing]
    E --> F[Visualization Engine]
    F --> G[Terminal UI]
    E --> H[Advisor Engine]
    H --> G
```

### Core Components

1. **Database Connector**: Handles connections to various database backends
2. **Plan Parser**: Parses and normalizes execution plans from different databases
3. **Visualization Engine**: Renders execution plans in the terminal
4. **Advisor Engine**: Analyzes plans and provides optimization suggestions
5. **TUI Interface**: Interactive terminal user interface

## 📚 Documentation

- [Architecture Decision Records](./docs/adr/README.md)
- [API Reference](https://docs.rs/sqltrace-rs)
- [Contributing Guide](./CONTRIBUTING.md)

## 🛣️ Roadmap

### Phase 1: MVP (PostgreSQL)

- [x] Project setup and basic structure
- [ ] PostgreSQL connection and query execution
- [ ] Basic TUI for plan visualization
- [ ] Support for EXPLAIN ANALYZE

### Phase 2: Advisor Engine

- [ ] Rule-based optimization suggestions
- [ ] Performance metrics and analysis
- [ ] Query history and comparison

### Phase 3: Multi-engine Support

- [ ] MySQL support
- [ ] SQLite support
- [ ] Abstract plan parser interface

### Phase 4: Advanced Features

- [ ] Export plans (JSON, HTML, PNG)
- [ ] Plugin system for custom visualizations
- [ ] Benchmarking tools

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 📜 License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

## 🙏 Acknowledgments

- Inspired by [pgAdmin](https://www.pgadmin.org/), [DBeaver](https://dbeaver.io/), and other database tools
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Uses [SQLx](https://github.com/launchbadge/sqlx) for database access
