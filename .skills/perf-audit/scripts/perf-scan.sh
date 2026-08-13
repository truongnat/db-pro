#!/usr/bin/env bash
# perf-scan.sh — Full-stack performance audit for DB Pro
# Usage: bash .skills/perf-audit/scripts/perf-scan.sh [section]
# Sections: frontend | er | rust | db | all (default: all)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'
BOLD='\033[1m'

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"
PASS=0
WARN=0
FAIL=0

section() {
  echo ""
  echo -e "${BLUE}${BOLD}═══ $1 ═══${NC}"
  echo ""
}

check() {
  local label="$1"
  local status="$2"  # pass, warn, fail
  local detail="$3"

  case "$status" in
    pass) echo -e "  ${GREEN}✓${NC} ${label}: ${detail}"; PASS=$((PASS + 1)) ;;
    warn) echo -e "  ${YELLOW}⚠${NC} ${label}: ${detail}"; WARN=$((WARN + 1)) ;;
    fail) echo -e "  ${RED}✗${NC} ${label}: ${detail}"; FAIL=$((FAIL + 1)) ;;
  esac
}

# Cross-platform file size getter
get_file_size() {
  local file="$1"
  if [[ "$OSTYPE" == "darwin"* ]]; then
    stat -f '%z' "$file" 2>/dev/null || echo 0
  else
    stat -c '%s' "$file" 2>/dev/null || echo 0
  fi
}

# ─── Frontend Bundle Analysis ───────────────────────────────────────

audit_frontend_bundle() {
  section "Frontend Bundle Analysis"

  local assets_dir="$PROJECT_ROOT/frontend/dist/assets"
  
  # Force rebuild if dist doesn't exist or is older than 1 hour
  local should_build=0
  if [ ! -d "$assets_dir" ]; then
    should_build=1
  else
    # Check if any JS file is older than 1 hour (3600 seconds)
    local now=$(date +%s)
    for f in "$assets_dir"/*.js; do
      if [ -f "$f" ]; then
        local mtime=$(stat -c %Y "$f" 2>/dev/null || stat -f %m "$f" 2>/dev/null || echo 0)
        local age=$((now - mtime))
        if [ "$age" -gt 3600 ]; then
          should_build=1
          break
        fi
      fi
    done
  fi

  if [ "$should_build" -eq 1 ]; then
    echo "  Building frontend (dist stale or missing)..."
    if ! (cd "$PROJECT_ROOT/frontend" && npm run build --silent 2>/dev/null); then
      check "Build" "fail" "npm run build failed"
      return
    fi
  fi

  # Total JS size
  local total_bytes=0
  for f in "$assets_dir"/*.js; do
    if [ -f "$f" ]; then
      local size=$(get_file_size "$f")
      total_bytes=$((total_bytes + size))
    fi
  done
  local total_mb=$(awk "BEGIN {printf \"%.2f\", $total_bytes / 1024 / 1024}")

  if [ "$total_bytes" -lt 1500000 ]; then
    check "Total JS size" "pass" "${total_mb}MB (target < 1.5MB)"
  elif [ "$total_bytes" -lt 2000000 ]; then
    check "Total JS size" "warn" "${total_mb}MB (target < 1.5MB, critical < 2MB)"
  else
    check "Total JS size" "fail" "${total_mb}MB exceeds 2MB critical threshold"
  fi

  # Largest chunk
  local largest_bytes=0
  local largest_name=""
  for f in "$assets_dir"/*.js; do
    if [ -f "$f" ]; then
      local size=$(get_file_size "$f")
      if [ "$size" -gt "$largest_bytes" ]; then
        largest_bytes=$size
        largest_name=$(basename "$f")
      fi
    fi
  done
  local largest_kb=$((largest_bytes / 1024))

  if [ "$largest_bytes" -lt 500000 ]; then
    check "Largest chunk" "pass" "${largest_kb}KB (${largest_name})"
  elif [ "$largest_bytes" -lt 800000 ]; then
    check "Largest chunk" "warn" "${largest_kb}KB (${largest_name})"
  else
    check "Largest chunk" "fail" "${largest_kb}KB (${largest_name}) exceeds 800KB"
  fi

  # CSS size
  local css_bytes=0
  for f in "$assets_dir"/*.css; do
    if [ -f "$f" ]; then
      local size=$(get_file_size "$f")
      css_bytes=$((css_bytes + size))
    fi
  done
  local css_kb=$((css_bytes / 1024))

  if [ "$css_bytes" -lt 100000 ]; then
    check "Total CSS" "pass" "${css_kb}KB"
  elif [ "$css_bytes" -lt 200000 ]; then
    check "Total CSS" "warn" "${css_kb}KB (target < 100KB)"
  else
    check "Total CSS" "fail" "${css_kb}KB exceeds 200KB"
  fi

  # Chunk count
  local chunk_count=0
  for f in "$assets_dir"/*.js; do
    if [ -f "$f" ]; then
      chunk_count=$((chunk_count + 1))
    fi
  done
  check "JS chunk count" "pass" "${chunk_count} chunks"
  
  # Verify code-splitting: check that vendor-reactflow and vendor-cytoscape are separate
  if ls "$assets_dir"/vendor-reactflow-*.js 1> /dev/null 2>&1 && \
     ls "$assets_dir"/vendor-cytoscape-*.js 1> /dev/null 2>&1; then
    check "ER code-splitting" "pass" "React Flow and Cytoscape in separate chunks"
  else
    check "ER code-splitting" "warn" "React Flow and Cytoscape may be bundled together"
  fi
}

# ─── Frontend Performance Budgets ──────────────────────────────────

audit_frontend_budgets() {
  section "Frontend Performance Budget Tests"

  if [ ! -d "$PROJECT_ROOT/frontend/node_modules" ]; then
    check "Dependencies" "fail" "Run npm install first"
    return
  fi

  local output
  if output=$(cd "$PROJECT_ROOT/frontend" && npx vitest run src/commons/__tests__/performance-budgets.test.ts --reporter=verbose 2>&1); then
    local test_count
    test_count=$(echo "$output" | grep -c "✓\|PASS\|passed" || true)
    check "Budget tests" "pass" "${test_count} tests passed"
  else
    local fail_count
    fail_count=$(echo "$output" | grep -c "✗\|FAIL\|failed" || true)
    check "Budget tests" "fail" "${fail_count} tests failed"
    echo "$output" | grep -E "✗|FAIL|AssertionError|expected|toBeLessThan" | head -10 | while read -r line; do
      echo "    $line"
    done
  fi
}

# ─── Rust Benchmarks ───────────────────────────────────────────────

audit_rust_bench() {
  section "Rust Backend Benchmarks"

  if ! command -v cargo &>/dev/null; then
    check "Cargo" "fail" "Rust toolchain not found"
    return
  fi

  if [ ! -f "$PROJECT_ROOT/crates/infrastructure/benches/sqlite_benchmarks.rs" ]; then
    check "Benchmarks" "warn" "No benchmark file found"
    return
  fi

  echo "  Running Criterion benchmarks (this may take a few minutes)..."
  local output
  if output=$(cd "$PROJECT_ROOT" && cargo bench --package db-pro-infrastructure -- --quick 2>&1); then
    check "Benchmarks" "pass" "All benchmarks completed"
    echo "$output" | grep -E "time:|thrpt:" | head -10 | while read -r line; do
      echo "    $line"
    done
  else
    check "Benchmarks" "fail" "Benchmark execution failed"
    echo "$output" | tail -5 | while read -r line; do
      echo "    $line"
    done
  fi
}

# ─── Rust Static Analysis ──────────────────────────────────────────

audit_rust_static() {
  section "Rust Static Analysis"

  # cargo check
  if (cd "$PROJECT_ROOT" && cargo check --workspace --quiet 2>&1); then
    check "cargo check" "pass" "Workspace compiles"
  else
    check "cargo check" "fail" "Compilation errors"
  fi

  # cargo clippy
  local clippy_out
  if clippy_out=$(cd "$PROJECT_ROOT" && cargo clippy --workspace --all-targets 2>&1); then
    local warn_count
    warn_count=$(echo "$clippy_out" | grep -c "warning:" || true)
    if [ "$warn_count" -eq 0 ]; then
      check "cargo clippy" "pass" "No warnings"
    else
      check "cargo clippy" "warn" "${warn_count} warnings"
    fi
  else
    check "cargo clippy" "fail" "Clippy errors"
  fi
}

# ─── Summary ───────────────────────────────────────────────────────

print_summary() {
  section "Audit Summary"
  echo -e "  ${GREEN}Passed: ${PASS}${NC}"
  echo -e "  ${YELLOW}Warnings: ${WARN}${NC}"
  echo -e "  ${RED}Failed: ${FAIL}${NC}"
  echo ""

  if [ "$FAIL" -gt 0 ]; then
    echo -e "  ${RED}${BOLD}Status: FAIL${NC} — ${FAIL} critical issue(s) found"
    return 1
  elif [ "$WARN" -gt 0 ]; then
    echo -e "  ${YELLOW}${BOLD}Status: WARN${NC} — ${WARN} warning(s), review recommended"
    return 0
  else
    echo -e "  ${GREEN}${BOLD}Status: PASS${NC} — All checks passed"
    return 0
  fi
}

# ─── Main ──────────────────────────────────────────────────────────

SECTION="${1:-all}"

echo -e "${BOLD}╔══════════════════════════════════════╗${NC}"
echo -e "${BOLD}║      DB Pro Performance Audit        ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════╝${NC}"

case "$SECTION" in
  frontend)
    audit_frontend_bundle
    audit_frontend_budgets
    ;;
  er)
    section "ER Diagram Performance"
    echo "  ER diagram performance requires runtime HUD inspection."
    echo "  This script cannot measure ER performance statically."
    echo ""
    echo "  Manual verification steps:"
    echo "    1. Open the app and navigate to a schema with 200+ tables"
    echo "    2. Open browser DevTools Console"
    echo "    3. Run: localStorage.setItem('er-perf-hud', '1')"
    echo "    4. Reload and observe the performance HUD"
    echo ""
    echo "  Reference targets:"
    echo "    200 tables: TTI < 2s, layout < 500ms, frame avg < 8ms"
    echo "    500 tables: TTI < 5s, layout < 1.5s, frame avg < 12ms"
    echo "   1000 tables: TTI < 10s, layout < 3s, frame avg < 16ms"
    echo ""
    check "ER performance" "warn" "Requires manual runtime verification"
    ;;
  rust)
    audit_rust_static
    audit_rust_bench
    ;;
  db)
    section "Database Query Performance"
    echo "  Database query performance requires a live connection."
    echo "  This script cannot measure query performance statically."
    echo ""
    echo "  Manual verification steps:"
    echo "    1. Connect to a PostgreSQL or SQLite database"
    echo "    2. Use the query editor to run:"
    echo "         EXPLAIN ANALYZE SELECT * FROM your_table WHERE condition;"
    echo ""
    echo "    3. Check for sequential scans on large tables (PostgreSQL):"
    echo "         SELECT schemaname, relname, seq_scan, seq_tup_read"
    echo "         FROM pg_stat_user_tables"
    echo "         WHERE seq_scan > 100 ORDER BY seq_tup_read DESC;"
    echo ""
    check "DB performance" "warn" "Requires live database connection"
    ;;
  all)
    audit_frontend_bundle
    audit_frontend_budgets
    audit_rust_static
    # Note: 'all' does not include rust benchmarks (too slow) or er/db (require runtime)
    echo ""
    echo "  Note: 'all' skips Rust benchmarks and ER/DB runtime checks."
    echo "  Run with 'rust' for benchmarks, or verify ER/DB manually."
    ;;
  *)
    echo "Usage: $0 [frontend|er|rust|db|all]"
    exit 1
    ;;
esac

print_summary
exit $?
