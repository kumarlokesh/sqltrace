# SQLTrace

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/sqltrace-rs.svg)](https://crates.io/crates/sqltrace-rs)

A high-performance, web-based SQL query analyzer that helps developers understand and optimize their database queries across PostgreSQL, MySQL, and SQLite.

## Features

- **Interactive query execution plan visualization**
- **Advanced benchmarking and performance analysis**
- **Intelligent optimization suggestions**
- **Multi-format export (JSON, HTML, text)**
- **RESTful API for integration**
- **Fast Rust backend with modern web UI**

## Quick Start

```bash
git clone https://github.com/kumarlokesh/sqltrace-rs.git
cd sqltrace-rs

# Start PostgreSQL with sample data
docker-compose up -d postgres

# Run SQLTrace
cargo run -- --database-url postgres://postgres:postgres@localhost:5432/sqltrace_dev
```

Open <http://localhost:3000> in your browser and start analyzing queries!

## Supported Databases

- **PostgreSQL** (full support with all features)
- **MySQL** (basic support, extensible)
- **SQLite** (basic support, extensible)

## Documentation

- [Setup Guide](docs/SETUP.md) - Detailed installation and configuration
- [API Reference](docs/API.md) - REST API documentation
- [Architecture](docs/ARCHITECTURE.md) - Technical architecture overview

## License

This project is licensed under the [MIT License](LICENSE).
