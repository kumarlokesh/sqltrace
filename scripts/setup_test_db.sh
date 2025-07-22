#!/bin/bash
# Script to set up the test database for sqltrace-rs using Docker

set -e

echo "🚀 Setting up test database for sqltrace-rs using Docker..."

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose not found. Please install Docker and docker-compose."
    exit 1
fi

# Start the database container
echo "🐳 Starting PostgreSQL container..."
docker-compose up -d postgres

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
for i in {1..10}; do
    if docker-compose exec -T postgres pg_isready -U postgres &> /dev/null; then
        break
    fi
    sleep 2
    echo "   Waiting for PostgreSQL to be ready... (attempt $i/10)"
done

# Verify connection
if ! docker-compose exec -T postgres pg_isready -U postgres &> /dev/null; then
    echo "❌ Could not connect to PostgreSQL container"
    docker-compose logs postgres
    exit 1
fi

# Create test database if it doesn't exist
echo "🔧 Setting up test database..."
docker-compose exec -T postgres psql -U postgres -c "CREATE DATABASE sqltrace_test;" 2> /dev/null || true

# Create .env file for tests
cat > tests/test.env <<EOL
# Test database configuration
TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/sqltrace_test
EOL

echo "✅ Test database setup complete!"
echo "   Test database URL: postgresql://postgres:****@localhost:5432/sqltrace_test"
echo ""
echo "To stop the test database, run: docker-compose down"
echo "   Configuration saved to tests/test.env"
