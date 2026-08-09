#!/usr/bin/env bash
# P2 Hardening — PostgreSQL fixture reset script
# Usage: ./reset.sh [connection_string]
# Default: postgresql://postgres:postgres@localhost:5432/dbpro_p2
#
# This script drops and recreates the P2 hardening fixture database.
# It is deterministic — running it multiple times produces the same result.

set -euo pipefail

CONNECTION_STRING="${1:-postgresql://postgres:postgres@localhost:5432/dbpro_p2}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== P2 Hardening Fixture Reset ==="
echo "Target: ${CONNECTION_STRING}"
echo ""

# Parse database name from connection string
DB_NAME="${CONNECTION_STRING##*/}"
DB_HOST_PART="${CONNECTION_STRING%%/*}"
DB_BASE="${DB_HOST_PART%%${DB_NAME}}"

echo "Step 1: Drop existing database (if exists)..."
psql "${DB_BASE}postgres" -c "DROP DATABASE IF EXISTS ${DB_NAME};" 2>/dev/null || true

echo "Step 2: Create database..."
psql "${DB_BASE}postgres" -c "CREATE DATABASE ${DB_NAME};"

echo "Step 3: Apply schema..."
psql "${CONNECTION_STRING}" -f "${SCRIPT_DIR}/001_schema.sql"

echo "Step 4: Seed data..."
psql "${CONNECTION_STRING}" -f "${SCRIPT_DIR}/002_seed.sql"

echo ""
echo "Step 5: Verify row counts..."
psql "${CONNECTION_STRING}" -c "
SELECT
    'organizations' AS table_name, count(*) FROM organizations
UNION ALL SELECT 'users', count(*) FROM users
UNION ALL SELECT 'profiles', count(*) FROM profiles
UNION ALL SELECT 'organization_members', count(*) FROM organization_members
UNION ALL SELECT 'categories', count(*) FROM categories
UNION ALL SELECT 'products', count(*) FROM products
UNION ALL SELECT 'product_categories', count(*) FROM product_categories
UNION ALL SELECT 'orders', count(*) FROM orders
UNION ALL SELECT 'order_items', count(*) FROM order_items
UNION ALL SELECT 'settings', count(*) FROM settings
UNION ALL SELECT 'audit_logs', count(*) FROM audit_logs
UNION ALL SELECT 'sessions', count(*) FROM sessions
UNION ALL SELECT 'documents', count(*) FROM documents
UNION ALL SELECT 'employees', count(*) FROM employees
ORDER BY table_name;
"

echo ""
echo "=== P2 fixture reset complete ==="
