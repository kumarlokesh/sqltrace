# Setup Guide

This guide covers detailed setup instructions for SQLTrace across different environments.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Docker](https://www.docker.com/get-started) and [docker-compose](https://docs.docker.com/compose/install/) (recommended)
- [just](https://github.com/casey/just) (optional, for development tasks)

## Database Setup

### Using Docker (Recommended)

Start all database engines with sample data:

```bash
# Start all database engines
docker-compose up -d

# Start specific engines only
docker-compose up -d postgres mysql

# Include database management UIs
docker-compose --profile admin up -d

# Access management interfaces:
# - pgAdmin: http://localhost:8080 (admin@sqltrace.dev / postgres)
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

## Installation

### From Source

```bash
git clone https://github.com/kumarlokesh/sqltrace-rs.git
cd sqltrace-rs
cargo build --release
```

### Usage

```bash
# PostgreSQL (recommended for full feature support)
sqltrace-rs --database-url postgres://postgres:postgres@localhost:5432/sqltrace_dev

# MySQL
sqltrace-rs --database-url mysql://mysql:mysql@localhost:3306/sqltrace_dev

# SQLite
sqltrace-rs --database-url sqlite:///path/to/database.db

# Custom host and port
sqltrace-rs --database-url postgres://user:password@localhost:5432/dbname --port 8080 --host 0.0.0.0
```

## Development Setup

### Running Tests

```bash
# Install just if you haven't
cargo install just

# Set up test database
docker-compose up -d postgres

# Run tests
just test

# Run tests with coverage
just coverage

# Clean up test database
just clean-test-db
```

### Development Workflow

1. Make your changes
2. Run tests: `just test`
3. Run lints: `just lint`
4. Format code: `cargo fmt`

## Connection Issues

Test database connections:

```bash
# Test PostgreSQL connection
psql postgres://postgres:postgres@localhost:5432/sqltrace_dev -c "SELECT 1"

# Test MySQL connection
mysql -h 127.0.0.1 -P 3306 -u mysql -pmysql sqltrace_dev -e "SELECT 1"

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
