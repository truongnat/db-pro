#!/usr/bin/env bash
set -euo pipefail

# Runs an executable subset of the PostgreSQL Sprint-1 test matrix and writes
# machine-friendly evidence logs under artifacts/pg-matrix.

DB_URL="${DATABASE_URL:-}"
OUT_DIR="${OUT_DIR:-artifacts/pg-matrix}"
REDACTED_DB_URL=""

usage() {
  cat <<'USAGE'
Usage:
  DATABASE_URL=postgresql://user:pass@host:5432/db ./scripts/pg_matrix_run.sh

Optional env:
  OUT_DIR=artifacts/pg-matrix

Notes:
  - Requires `psql` in PATH.
  - Script performs non-destructive DDL on table `dbpro_lock_test`.
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ -z "$DB_URL" ]]; then
  echo "Missing DATABASE_URL." >&2
  usage
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "psql is required but was not found in PATH." >&2
  exit 1
fi

REDACTED_DB_URL="$(echo "$DB_URL" | sed -E 's#(postgres(ql)?://[^:/@]+):[^@]*@#\1:***@#')"

mkdir -p "$OUT_DIR"
timestamp="$(date +%Y%m%d-%H%M%S)"
log_file="$OUT_DIR/pg-matrix-$timestamp.log"
summary_file="$OUT_DIR/pg-matrix-$timestamp-summary.md"
locker_file="$OUT_DIR/pg-matrix-locker-$timestamp.log"

pass_count=0
fail_count=0

run_case() {
  local case_id="$1"
  local title="$2"
  local expected="$3"
  local sql="$4"
  local expect_substring="${5:-}"

  local started ended elapsed_ms output rc status
  started="$(python3 -c 'import time; print(int(time.time()*1000))')"
  set +e
  output="$(psql "$DB_URL" -X -v ON_ERROR_STOP=1 -tA -c "$sql" 2>&1)"
  rc=$?
  set -e
  ended="$(python3 -c 'import time; print(int(time.time()*1000))')"
  elapsed_ms="$((ended - started))"

  status="FAIL"
  if [[ "$expected" == "success" && "$rc" -eq 0 ]]; then
    status="PASS"
  elif [[ "$expected" == "failure" && "$rc" -ne 0 ]]; then
    if [[ -n "$expect_substring" ]]; then
      if grep -qi -- "$expect_substring" <<<"$output"; then
        status="PASS"
      fi
    else
      status="PASS"
    fi
  fi

  if [[ "$status" == "PASS" ]]; then
    pass_count=$((pass_count + 1))
  else
    fail_count=$((fail_count + 1))
  fi

  {
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $case_id $title"
    echo "expected=$expected rc=$rc elapsed_ms=$elapsed_ms status=$status"
    if [[ -n "$expect_substring" ]]; then
      echo "expect_substring=$expect_substring"
    fi
    echo "output_begin"
    echo "$output"
    echo "output_end"
    echo
  } >>"$log_file"

  printf '| %s | %s | %s | %s | %s |\n' "$case_id" "$title" "$status" "$elapsed_ms" "$expected" >>"$summary_file"
}

run_lock_timeout_case() {
  local case_id="PGM-06"
  local title="Lock timeout under contention"
  local expected="failure"
  local expect_substring="lock timeout"

  # Setup table once, non-destructive.
  psql "$DB_URL" -X -v ON_ERROR_STOP=1 -c \
    "CREATE TABLE IF NOT EXISTS dbpro_lock_test(id integer PRIMARY KEY, payload text);" \
    >/dev/null

  # Acquire exclusive lock in background session.
  psql "$DB_URL" -X -v ON_ERROR_STOP=1 -c \
    "BEGIN; LOCK TABLE dbpro_lock_test IN ACCESS EXCLUSIVE MODE; SELECT pg_sleep(12); COMMIT;" \
    >"$locker_file" 2>&1 &
  local locker_pid=$!

  sleep 1

  local started ended elapsed_ms output rc status
  started="$(python3 -c 'import time; print(int(time.time()*1000))')"
  set +e
  output="$(psql "$DB_URL" -X -v ON_ERROR_STOP=1 -c \
    "SET lock_timeout = '1500ms'; ALTER TABLE dbpro_lock_test ADD COLUMN IF NOT EXISTS matrix_marker text;" \
    2>&1)"
  rc=$?
  set -e
  ended="$(python3 -c 'import time; print(int(time.time()*1000))')"
  elapsed_ms="$((ended - started))"

  if kill -0 "$locker_pid" >/dev/null 2>&1; then
    wait "$locker_pid" || true
  fi

  status="FAIL"
  if [[ "$rc" -ne 0 ]] && grep -qi -- "$expect_substring" <<<"$output"; then
    status="PASS"
  fi

  if [[ "$status" == "PASS" ]]; then
    pass_count=$((pass_count + 1))
  else
    fail_count=$((fail_count + 1))
  fi

  {
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $case_id $title"
    echo "expected=$expected rc=$rc elapsed_ms=$elapsed_ms status=$status"
    echo "expect_substring=$expect_substring"
    echo "output_begin"
    echo "$output"
    echo "output_end"
    echo "locker_log_file=$locker_file"
    echo
  } >>"$log_file"

  printf '| %s | %s | %s | %s | %s |\n' "$case_id" "$title" "$status" "$elapsed_ms" "$expected" >>"$summary_file"
}

{
  echo "# PostgreSQL Matrix Run"
  echo
  echo "- Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- Target: ${REDACTED_DB_URL%%\?*}"
  echo "- Log file: \`$log_file\`"
  echo
  echo "| Case ID | Scenario | Result | Elapsed (ms) | Expected |"
  echo "|---|---|---|---:|---|"
} >"$summary_file"

run_case "PGM-01" "Connectivity ping" "success" "SELECT 1;"
run_case "PGM-02" "Version introspection" "success" "SHOW server_version;"
run_case "PGM-03" "Statement timeout success path" "success" "SET statement_timeout = '4000ms'; SELECT pg_sleep(1);"
run_case "PGM-04" "Statement timeout enforcement" "failure" "SET statement_timeout = '1000ms'; SELECT pg_sleep(5);" "statement timeout"
run_case "PGM-05" "Wrapped paging/filter/sort compatibility" "success" \
  "SELECT * FROM (SELECT generate_series(1, 2000) AS id) AS __db_pro_page WHERE LOWER(CAST(__db_pro_page.\"id\" AS TEXT)) LIKE '%9%' ESCAPE '\\' ORDER BY __db_pro_page.\"id\" ASC LIMIT 501 OFFSET 0;"
run_lock_timeout_case

{
  echo
  echo "## Totals"
  echo
  echo "- Passed: $pass_count"
  echo "- Failed: $fail_count"
} >>"$summary_file"

echo "Matrix completed."
echo "Summary: $summary_file"
echo "Detail log: $log_file"

if [[ "$fail_count" -gt 0 ]]; then
  exit 2
fi
