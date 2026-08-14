#!/usr/bin/env bash
# Smoke fixture verification script.
# Verifies object counts and representative rows for both providers.
#
# Usage:
#   ./verify-smoke.sh postgres [PG_CONN_STRING]
#   ./verify-smoke.sh sqlite [SQLITE_DB_PATH]
#
# Exit 0 on success, non-zero on any assertion failure.

set -euo pipefail

PROVIDER="${1:-}"
PASS=0
FAIL=0

assert_count() {
  local label="$1" expected="$2" actual="$3"
  if [ "$actual" = "$expected" ]; then
    echo "  ✓ $label: $actual (expected $expected)"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $label: $actual (expected $expected)"
    FAIL=$((FAIL + 1))
  fi
}

if [ -z "$PROVIDER" ]; then
  echo "Usage: $0 postgres|sqlite [connection]"
  exit 1
fi

echo "=== Smoke fixture verification: $PROVIDER ==="

if [ "$PROVIDER" = "postgres" ]; then
  PG="${2:-postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture}"
  echo "Connection: $PG"

  run_sql() { psql -qAt -d "$PG" -c "$1" | tr -d '[:space:]'; }

  assert_count "tables"    "10" "$(run_sql "SELECT count(*) FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'smoke_%' AND table_type='BASE TABLE'")"
  assert_count "views"     "2"  "$(run_sql "SELECT count(*) FROM information_schema.views WHERE table_schema='public' AND table_name LIKE 'smoke_%'")"
  assert_count "indexes"   "8"  "$(run_sql "SELECT count(*) FROM pg_indexes WHERE schemaname='public' AND indexname LIKE 'idx_smoke_%'")"
  assert_count "triggers"  "1"  "$(run_sql "SELECT count(*) FROM pg_trigger WHERE tgname='smoke_users_updated_at'")"
  assert_count "users"     "5"  "$(run_sql "SELECT count(*) FROM smoke_users")"
  assert_count "orders"    "8"  "$(run_sql "SELECT count(*) FROM smoke_orders")"
  assert_count "items"     "10" "$(run_sql "SELECT count(*) FROM smoke_order_items")"
  assert_count "audit"     "5"  "$(run_sql "SELECT count(*) FROM smoke_audit_log")"
  assert_count "documents" "3"  "$(run_sql "SELECT count(*) FROM smoke_documents")"
  assert_count "employees" "3"  "$(run_sql "SELECT count(*) FROM smoke_employees")"
  assert_count "products"  "5"  "$(run_sql "SELECT count(*) FROM smoke_products")"
  assert_count "network"   "3"  "$(run_sql "SELECT count(*) FROM smoke_network")"

elif [ "$PROVIDER" = "sqlite" ]; then
  DB="${2:-fixtures/smoke/sqlite/smoke.db}"
  echo "Database: $DB"

  if [ ! -f "$DB" ]; then
    echo "  Database file not found: $DB"
    echo "  Run: sqlite3 $DB < fixtures/smoke/sqlite/smoke_fixture.sql"
    exit 1
  fi

  run_sql() { sqlite3 "$DB" "$1" | tr -d '[:space:]'; }

  assert_count "tables"    "10" "$(run_sql "SELECT count(*) FROM sqlite_master WHERE type='table' AND name LIKE 'smoke_%'")"
  assert_count "views"     "2"  "$(run_sql "SELECT count(*) FROM sqlite_master WHERE type='view' AND name LIKE 'smoke_%'")"
  assert_count "indexes"   "6"  "$(run_sql "SELECT count(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_smoke_%'")"
  assert_count "triggers"  "1"  "$(run_sql "SELECT count(*) FROM sqlite_master WHERE type='trigger' AND name LIKE 'smoke_%'")"
  assert_count "users"     "5"  "$(run_sql "SELECT count(*) FROM smoke_users")"
  assert_count "orders"    "8"  "$(run_sql "SELECT count(*) FROM smoke_orders")"
  assert_count "items"     "10" "$(run_sql "SELECT count(*) FROM smoke_order_items")"
  assert_count "audit"     "5"  "$(run_sql "SELECT count(*) FROM smoke_audit_log")"
  assert_count "documents" "3"  "$(run_sql "SELECT count(*) FROM smoke_documents")"
  assert_count "employees" "3"  "$(run_sql "SELECT count(*) FROM smoke_employees")"
  assert_count "products"  "5"  "$(run_sql "SELECT count(*) FROM smoke_products")"
  assert_count "network"   "3"  "$(run_sql "SELECT count(*) FROM smoke_network")"
else
  echo "Unknown provider: $PROVIDER"
  exit 1
fi

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
