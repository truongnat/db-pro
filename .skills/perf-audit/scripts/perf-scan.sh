#!/usr/bin/env bash
# perf-scan.sh — Full-stack performance audit for DB Pro
# Usage: bash .skills/perf-audit/scripts/perf-scan.sh [section]
# Sections: native | er | rust | db | all (default: all)
#
# The product UI is native eframe/egui (crates/ui + crates/native-app).
# There is no JS bundle, no React, and no Node toolchain in this repository.
# The React frontend is archived under _archive/frontend/.

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

# ─── Native UI Binary ──────────────────────────────────────────────

audit_native_binary() {
  section "Native UI Binary"

  if ! command -v cargo &>/dev/null; then
    check "Cargo" "fail" "Rust toolchain not found"
    return
  fi

  echo "  Building db-pro-native (release)..."
  local output
  if output=$(cd "$PROJECT_ROOT" && cargo build --release --locked -p db-pro-native 2>&1); then
    check "cargo build -p db-pro-native" "pass" "release build succeeded"
  else
    check "cargo build -p db-pro-native" "fail" "release build failed"
    echo "$output" | tail -5 | while read -r line; do
      echo "    $line"
    done
    return
  fi

  local bin="$PROJECT_ROOT/target/release/db-pro-native"
  if [ ! -f "$bin" ]; then
    check "Binary artifact" "fail" "target/release/db-pro-native not found"
    return
  fi

  local bytes
  bytes=$(get_file_size "$bin")
  local mb
  mb=$(awk "BEGIN {printf \"%.1f\", $bytes / 1024 / 1024}")

  if [ "$bytes" -lt 52428800 ]; then
    check "Binary size" "pass" "${mb}MB (target < 50MB)"
  elif [ "$bytes" -lt 104857600 ]; then
    check "Binary size" "warn" "${mb}MB (target < 50MB, critical < 100MB)"
  else
    check "Binary size" "fail" "${mb}MB exceeds 100MB critical threshold"
  fi
}

# ─── Native UI Benchmarks ──────────────────────────────────────────

audit_native_benches() {
  section "Native UI Benchmarks (criterion)"

  local bench_file="$PROJECT_ROOT/crates/ui/benches/result_grid_benchmarks.rs"
  if [ ! -f "$bench_file" ]; then
    check "UI benchmarks" "warn" "crates/ui/benches/result_grid_benchmarks.rs not found"
    return
  fi

  echo "  Running criterion benchmarks for db-pro-ui (this may take a few minutes)..."
  local output
  if output=$(cd "$PROJECT_ROOT" && cargo bench --package db-pro-ui -- --quick 2>&1); then
    check "UI benchmarks" "pass" "All benchmarks completed"
    echo "$output" | grep -E "time:" | head -10 | while read -r line; do
      echo "    $line"
    done
  else
    check "UI benchmarks" "fail" "Benchmark execution failed"
    echo "$output" | tail -5 | while read -r line; do
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
  native)
    audit_native_binary
    audit_native_benches
    ;;
  er)
    section "ER Diagram Performance"
    echo "  ER diagram performance requires runtime measurement of the native painter"
    echo "  in crates/ui (diagram_view.rs); it cannot be measured statically."
    echo ""
    echo "  Manual verification steps:"
    echo "    1. Run: RUST_LOG=db_pro_ui=debug cargo run -p db-pro-native"
    echo "    2. Open a schema with 200+ tables and open the ER diagram tab"
    echo "    3. Record layout time, pan/zoom frame time, and time-to-interactive"
    echo ""
    echo "  Reference targets:"
    echo "    200 tables: TTI < 2s, layout < 500ms, frame avg < 8ms"
    echo "    500 tables: TTI < 5s, layout < 1.5s, frame avg < 12ms"
    echo "    1000 tables: TTI < 10s, layout < 3s, frame avg < 16ms"
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
    echo "    1. Connect to a PostgreSQL or SQLite database in db-pro-native"
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
    audit_native_binary
    audit_rust_static
    # Note: 'all' does not include benchmarks (too slow) or er/db (require runtime)
    echo ""
    echo "  Note: 'all' skips benchmarks and ER/DB runtime checks."
    echo "  Run with 'native' or 'rust' for benchmarks, or verify ER/DB manually."
    ;;
  *)
    echo "Usage: $0 [native|er|rust|db|all]"
    exit 1
    ;;
esac

print_summary
exit $?
