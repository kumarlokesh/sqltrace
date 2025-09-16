# Justfile for sqltrace-rs development tasks

default:
    @just --list

# Development Environment Setup
# ============================

# Set up development environment (database, dependencies, etc.)
setup:
    @echo "🚀 Setting up SQLTrace development environment..."
    cp .env.example .env
    just db-up
    just db-wait
    @echo "✅ Development environment ready!"

# Start PostgreSQL database container (primary)
db-up:
    @echo "🐘 Starting PostgreSQL container..."
    docker-compose up -d postgres

# Start all database engines
db-up-all:
    @echo "🗄️ Starting all database engines..."
    docker-compose up -d postgres mysql sqlite

# Stop database containers
db-down:
    @echo "🛑 Stopping database containers..."
    docker-compose down

# Restart PostgreSQL database
db-restart: db-down db-up

# Wait for PostgreSQL to be ready
db-wait:
    #!/usr/bin/env bash
    echo "⏳ Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if docker-compose exec postgres pg_isready -U postgres >/dev/null 2>&1; then
            echo "✅ PostgreSQL is ready!"
            exit 0
        fi
        echo "Waiting... ($i/30)"
        sleep 1
    done
    echo "❌ PostgreSQL failed to start within 30 seconds"
    exit 1

# View database logs
db-logs:
    docker-compose logs -f postgres

# Start pgAdmin (optional database management UI)
pgadmin-up:
    @echo "🔧 Starting pgAdmin..."
    docker-compose --profile admin up -d pgadmin
    @echo "📊 pgAdmin available at http://localhost:8080"
    @echo "   Email: admin@sqltrace.dev"
    @echo "   Password: postgres"

# Connect to PostgreSQL with psql
db-connect:
    docker-compose exec postgres psql -U postgres -d sqltrace_dev

# Connect to MySQL
mysql-connect:
    docker-compose exec mysql mysql -u mysql -pmysql sqltrace_dev

# Connect to SQLite
sqlite-connect:
    docker-compose exec sqlite sqlite3 /data/sqltrace_dev.db

# Development & Testing
# =====================

# Run the application in development mode with PostgreSQL
dev:
    @echo "🚀 Starting SQLTrace web server..."
    @echo "📊 Open http://localhost:3000 in your browser"
    cargo run -- --database-url "postgres://postgres:postgres@localhost:5432/sqltrace_dev"

# Run with MySQL
dev-mysql: 
    @echo "🚀 Starting SQLTrace with MySQL..."
    @echo "📊 Open http://localhost:3000 in your browser"
    cargo run -- --database-url "mysql://mysql:mysql@localhost:3306/sqltrace_dev"

# Run with SQLite
dev-sqlite:
    @echo "🚀 Starting SQLTrace with SQLite..."
    @echo "📊 Open http://localhost:3000 in your browser"
    cargo run -- --database-url "sqlite:///tmp/sqltrace_dev.db"

# Run all tests (unit + integration)
test: db-wait
    @echo "🧪 Running tests..."
    DATABASE_URL="postgres://postgres:postgres@localhost:5432/sqltrace_dev" cargo test

# Run only unit tests (no database required)
test-unit:
    @echo "🧪 Running unit tests..."
    cargo test --lib

# Run integration tests with database
test-integration: db-wait
    @echo "🧪 Running integration tests..."
    DATABASE_URL="postgres://postgres:postgres@localhost:5432/sqltrace_dev" cargo test --test integration_test

# Run tests with coverage (requires cargo-tarpaulin)
coverage: db-wait
    @echo "📊 Running tests with coverage..."
    DATABASE_URL="postgres://postgres:postgres@localhost:5432/sqltrace_dev" cargo tarpaulin --out Html

# Build the project
build:
    @echo "🔨 Building SQLTrace..."
    cargo build

# Build release version
build-release:
    @echo "🔨 Building SQLTrace (release)..."
    cargo build --release

# Code Quality & Formatting
# =========================

# Run all lints and checks
check: lint test-unit
    @echo "✅ All checks passed!"

# Run lints and format checks
lint:
    @echo "🔍 Running lints..."
    cargo clippy -- -D warnings
    cargo fmt -- --check

# Fix formatting
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt

# Fix lints where possible
fix:
    @echo "🔧 Fixing lints..."
    cargo clippy --fix --allow-dirty
    cargo fmt

# Database Management
# ===================

# Reset database with fresh data
db-reset: db-down db-up db-wait
    @echo "🔄 Database reset complete!"

# Show database status
db-status:
    @echo "📊 Database Status:"
    @docker-compose ps
    @echo ""
    @echo "🐘 PostgreSQL: postgres://postgres:postgres@localhost:5432/sqltrace_dev"
    @echo "🐬 MySQL:      mysql://mysql:mysql@localhost:3306/sqltrace_dev"
    @echo "🗄️ SQLite:     /data/sqltrace_dev.db (in container)"

# Example Queries
# ===============

# Show example queries for testing
examples:
    @echo "📝 Example SQL queries to test in SQLTrace:"
    @echo ""
    @echo "Simple query:"
    @echo "  SELECT * FROM ecommerce.users LIMIT 10;"
    @echo ""
    @echo "Join with aggregation:"
    @echo "  SELECT u.username, COUNT(o.id) as order_count"
    @echo "  FROM ecommerce.users u"
    @echo "  LEFT JOIN ecommerce.orders o ON u.id = o.user_id"
    @echo "  GROUP BY u.id, u.username"
    @echo "  ORDER BY order_count DESC;"
    @echo ""
    @echo "Complex query with multiple joins:"
    @echo "  SELECT p.name, c.name as category, AVG(oi.quantity) as avg_quantity"
    @echo "  FROM ecommerce.products p"
    @echo "  JOIN ecommerce.categories c ON p.category_id = c.id"
    @echo "  LEFT JOIN ecommerce.order_items oi ON p.id = oi.product_id"
    @echo "  GROUP BY p.id, p.name, c.name"
    @echo "  HAVING AVG(oi.quantity) > 1.0"
    @echo "  ORDER BY avg_quantity DESC;"
    @echo ""
    @echo "🎯 Open http://localhost:3000 and paste these queries to analyze their execution plans!"
