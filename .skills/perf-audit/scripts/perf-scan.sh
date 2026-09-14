#!/usr/bin/env bash
# perf-scan.sh — Full-stack performance audit for DB Pro
# Usage: bash .skills/perf-audit/scripts/perf-scan.sh [section]
# Sections: native | er | rust | db | all (default: all)
# Flags:    --self-test  (assert the result/exit-code contract; no build, no benchmark)
#
# The product UI is native eframe/egui (crates/ui + crates/native-app).
# There is no JS bundle, no React, and no Node toolchain in this repository.
# The React frontend is archived under _archive/frontend/.
#
# Result semantics (full contract in .skills/perf-audit/SKILL.md §8):
#   PASS  exit 0 — every *executed* check passed.  When a section was deliberately not
#                  executed the status reads "PASS (partial)" and names those sections, so
#                  "PASS" can never be quoted as "everything was measured".  A run whose
#                  working tree is not committed is qualified with "+dirty(N)": the numbers
#                  describe the tree, not the recorded commit.
#   WARN  exit 2 — at least one warning and no failure.  A warning is never a pass: an
#                  automated gate that only reads the exit status cannot certify it.
#   FAIL  exit 1 — at least one failed check.
#
# Provenance: every run prints the source revision it measured and the sha256 of the
# measured binary, so a recorded result can be tied to a tree and to an artifact.  The
# size check measures the binary produced by the build in this same run; when the build
# fails, no size is reported at all (a stale binary is never measured).

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
SKIPPED=0
SKIPPED_LABELS=""

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

# A section that was not executed is counted and named, never silently omitted.
skip() {
  local label="$1"
  local reason="$2"
  echo -e "  ${BLUE}•${NC} ${label}: not executed — ${reason}"
  SKIPPED=$((SKIPPED + 1))
  if [ -z "$SKIPPED_LABELS" ]; then
    SKIPPED_LABELS="$label"
  else
    SKIPPED_LABELS="$SKIPPED_LABELS, $label"
  fi
}

# Source revision of the tree the scan runs against, with a dirty marker when the
# working tree is not committed.
source_revision() {
  local sha dirty
  sha="$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || true)"
  if [ -z "$sha" ]; then
    echo "unknown"
    return
  fi
  dirty="$(git -C "$PROJECT_ROOT" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
  if [ "${dirty:-0}" -gt 0 ]; then
    echo "${sha}+dirty(${dirty})"
  else
    echo "$sha"
  fi
}

SOURCE_REVISION="$(source_revision)"
SOURCE_IS_DIRTY=0
case "$SOURCE_REVISION" in
  *+dirty*) SOURCE_IS_DIRTY=1 ;;
esac

# sha256 of the measured artifact (used to tie a recorded size to a binary).
sha256_of() {
  local file="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" 2>/dev/null | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" 2>/dev/null | awk '{print $1}'
  else
    echo "unavailable"
  fi
}

BINARY_DIGEST="unavailable"
MEASURED_ARTIFACT="not measured in this run"

# Cross-platform file size getter
get_file_size() {
  local file="$1"
  if [[ "$OSTYPE" == "darwin"* ]]; then
    stat -f '%z' "$file" 2>/dev/null || echo 0
  else
    stat -c '%s' "$file" 2>/dev/null || echo 0
  fi
}

# ─── Result contract (shared by print_summary and --self-test) ─────

status_line() {
  local line
  if [ "$FAIL" -gt 0 ]; then
    line="FAIL — ${FAIL} failed, ${PASS} passed"
  elif [ "$WARN" -gt 0 ]; then
    line="WARN — ${WARN} warning(s), ${PASS} passed"
  elif [ "$SKIPPED" -gt 0 ]; then
    line="PASS (partial) — ${PASS} executed check(s) passed; not run: ${SKIPPED_LABELS}"
  else
    line="PASS — ${PASS} executed check(s) passed"
  fi
  if [ "$SOURCE_IS_DIRTY" -eq 1 ]; then
    line="${line} [source ${SOURCE_REVISION}: working tree not committed]"
  fi
  echo "$line"
}

status_exit_code() {
  if [ "$FAIL" -gt 0 ]; then
    echo 1
  elif [ "$WARN" -gt 0 ]; then
    echo 2
  else
    echo 0
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
    # No size claim at all: measuring a binary the build did not produce is exactly the
    # stale-artifact case the result semantics forbid.
    echo "  (no binary size reported — the artifact of this run was not produced)"
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

  BINARY_DIGEST="$(sha256_of "$bin")"
  MEASURED_ARTIFACT="target/release/db-pro-native sha256 ${BINARY_DIGEST} (${mb}MB)"

  if [ "$bytes" -lt 52428800 ]; then
    check "Binary size" "pass" "${mb}MB (target < 50MB) sha256 ${BINARY_DIGEST:0:16}…"
  elif [ "$bytes" -lt 104857600 ]; then
    check "Binary size" "warn" "${mb}MB (target < 50MB, critical < 100MB) sha256 ${BINARY_DIGEST:0:16}…"
  else
    check "Binary size" "fail" "${mb}MB exceeds 100MB critical threshold sha256 ${BINARY_DIGEST:0:16}…"
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

# ─── Self-test ─────────────────────────────────────────────────────

SELF_TEST_FAILURES=0

assert_eq() {
  local expected="$1"
  local actual="$2"
  local label="$3"

  if [ "$expected" = "$actual" ]; then
    echo -e "  ${GREEN}✓${NC} ${label}"
  else
    echo -e "  ${RED}✗${NC} ${label}: expected '${expected}', got '${actual}'"
    SELF_TEST_FAILURES=$((SELF_TEST_FAILURES + 1))
  fi
}

run_self_test() {
  section "Self-test — result and exit-code contract"

  local saved_pass=$PASS saved_warn=$WARN saved_fail=$FAIL
  local saved_skipped=$SKIPPED saved_labels="$SKIPPED_LABELS" saved_dirty=$SOURCE_IS_DIRTY

  PASS=3; WARN=0; FAIL=0; SKIPPED=0; SKIPPED_LABELS=""; SOURCE_IS_DIRTY=0
  assert_eq "PASS — 3 executed check(s) passed" "$(status_line)" "clean full pass reads as an unqualified PASS"
  assert_eq 0 "$(status_exit_code)" "clean full pass exits 0"

  SOURCE_IS_DIRTY=1
  assert_eq 1 "$(status_line | grep -c 'working tree not committed')" "an uncommitted tree is named in the status line"
  assert_eq 0 "$(status_exit_code)" "an uncommitted tree does not fail the scan"
  SOURCE_IS_DIRTY=0

  WARN=1
  assert_eq 1 "$(status_line | grep -c '^WARN')" "a warning is reported as WARN"
  assert_eq 2 "$(status_exit_code)" "a warning exits 2, never 0"
  WARN=0

  FAIL=1
  assert_eq 1 "$(status_line | grep -c '^FAIL')" "a failure is reported as FAIL"
  assert_eq 1 "$(status_exit_code)" "a failure exits 1"
  FAIL=0

  PASS=4; SKIPPED=2; SKIPPED_LABELS="ER diagram runtime, DB query performance"
  assert_eq 1 "$(status_line | grep -c 'PASS (partial)')" "skipped sections make the pass partial"
  assert_eq 1 "$(status_line | grep -c 'ER diagram runtime')" "skipped sections are named"
  assert_eq 0 "$(status_exit_code)" "a partial pass still exits 0"

  SKIPPED=0; SKIPPED_LABELS=""
  skip "probe" "self-test" >/dev/null
  assert_eq 1 "$SKIPPED" "skip() counts a section as not executed"
  assert_eq "probe" "$SKIPPED_LABELS" "skip() records the section name"
  SKIPPED=0; SKIPPED_LABELS=""

  local tmp expected actual
  tmp="$(mktemp)"
  printf 'perf-scan-self-test\n' > "$tmp"
  actual="$(sha256_of "$tmp")"
  if command -v openssl >/dev/null 2>&1; then
    expected="$(openssl dgst -sha256 "$tmp" | awk '{print $NF}')"
    assert_eq "$expected" "$actual" "sha256_of matches openssl on a known file"
  else
    assert_eq 64 "${#actual}" "sha256_of returns a 64-character digest"
  fi
  rm -f "$tmp"

  PASS=$saved_pass; WARN=$saved_warn; FAIL=$saved_fail
  SKIPPED=$saved_skipped; SKIPPED_LABELS="$saved_labels"; SOURCE_IS_DIRTY=$saved_dirty

  echo ""
  if [ "$SELF_TEST_FAILURES" -eq 0 ]; then
    echo -e "  ${GREEN}${BOLD}SELF-TEST PASS${NC} — result and exit-code contract holds"
    return 0
  fi
  echo -e "  ${RED}${BOLD}SELF-TEST FAIL${NC} — ${SELF_TEST_FAILURES} assertion(s) failed"
  return 1
}

# ─── Summary ───────────────────────────────────────────────────────

print_summary() {
  local code
  section "Audit Summary"
  echo -e "  Source revision: ${SOURCE_REVISION}"
  echo -e "  Measured artifact: ${MEASURED_ARTIFACT}"
  echo ""
  echo -e "  ${GREEN}Passed: ${PASS}${NC}"
  echo -e "  ${YELLOW}Warnings: ${WARN}${NC}"
  echo -e "  ${RED}Failed: ${FAIL}${NC}"
  echo -e "  ${BLUE}Not executed: ${SKIPPED}${NC}"
  echo ""
  echo -e "  Status: $(status_line)"
  echo ""

  code="$(status_exit_code)"
  if [ "$code" = "1" ]; then
    echo -e "  ${RED}${BOLD}${FAIL} critical issue(s) found.${NC}"
  elif [ "$code" = "2" ]; then
    echo -e "  ${YELLOW}${BOLD}Review recommended — warnings are not a pass (exit 2).${NC}"
  fi
  return "$code"
}

# ─── Main ──────────────────────────────────────────────────────────

SECTION="${1:-all}"

echo -e "${BOLD}╔══════════════════════════════════════╗${NC}"
echo -e "${BOLD}║      DB Pro Performance Audit        ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════╝${NC}"
echo -e "  Source revision: ${SOURCE_REVISION}"

case "$SECTION" in
  --self-test)
    run_self_test
    exit $?
    ;;
  native)
    audit_native_binary
    audit_native_benches
    ;;
  er)
    section "ER Diagram Performance"
    echo "  ER diagram performance requires runtime measurement of the native painter"
    echo "  in crates/ui (diagram_view.rs + diagram/); it cannot be measured statically."
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
    # 'all' cannot run the benchmark and runtime sections: benchmarks are too slow for the
    # default gate, ER needs a window server and DB needs a live connection. They are
    # counted and named as not executed instead of being folded into a bare PASS.
    skip "Native UI benchmarks" "run section 'native'"
    skip "Rust backend benchmarks" "run section 'rust'"
    skip "ER diagram runtime" "needs a window server; run section 'er'"
    skip "DB query performance" "needs a live connection; run section 'db'"
    ;;
  *)
    echo "Usage: $0 [native|er|rust|db|all|--self-test]"
    exit 1
    ;;
esac

print_summary
exit $?
