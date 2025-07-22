# Justfile for sqltrace-rs development tasks

default:
    @just --list

# Set up test database using Docker
setup-test-db:
    #!/usr/bin/env bash
    set -e
    chmod +x ./scripts/setup_test_db.sh
    ./scripts/setup_test_db.sh

# Start test database container
db-up:
    #!/usr/bin/env bash
    docker-compose up -d postgres

# Stop test database container
db-down:
    #!/usr/bin/env bash
    docker-compose down

# View test database logs
db-logs:
    #!/usr/bin/env bash
    docker-compose logs -f postgres

# Run tests with a clean test database
test:
    #!/usr/bin/env bash
    set -e
    
    # Load test environment
    if [ -f tests/test.env ]; then
        export $(grep -v '^#' tests/test.env | xargs)
    fi
    
    # Run tests with single thread to avoid DB conflicts
    cargo test -- --test-threads=1

# Run tests with coverage (requires cargo-tarpaulin)
coverage:
    #!/usr/bin/env bash
    set -e
    
    # Load test environment
    if [ -f tests/test.env ]; then
        export $(grep -v '^#' tests/test.env | xargs)
    fi
    
    cargo tarpaulin -- --test-threads=1

# Run lints
lint:
    cargo clippy -- -D warnings
    cargo fmt -- --check

# Clean up test database
clean-test-db:
    #!/usr/bin/env bash
    set -e
    
    # Load test environment
    if [ -f tests/test.env ]; then
        export $(grep -v '^#' tests/test.env | xargs)
    else
        echo "No test.env file found. Using defaults."
        export TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/sqltrace_test"
    fi
    
    # Extract database name from URL
    DB_NAME=$(echo $TEST_DATABASE_URL | sed -E 's/.*\/([^?]+)(\?.*)?/\1/')
    
    echo "Dropping test database: $DB_NAME"
    dropdb --if-exists "$DB_NAME"
    echo "Test database dropped"
