#!/usr/bin/env bash
# Deterministic UI evidence capture matrix — audit 2026-10-13.
# Requires: cargo build -p db-pro-native --features capture
# Usage:    bash docs/plans/_audit-2026-10-13/capture.sh [out_dir] [sizes...]
# Self-terminating: the driver exits after the screenshot or the frame timeout.

set -u
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)"

BIN="target/debug/db-pro-native"
OUT="${1:-docs/plans/_audit-2026-10-13/captures}"
shift 1 2>/dev/null || true
SIZES=("${@:-1280x800}")
[ "${#SIZES[@]}" -eq 0 ] && SIZES=(1280x800)
mkdir -p "$OUT"
LOG="$OUT/capture.log"
: > "$LOG"

cap() { # cap <name> <env...>
  local name="$1"; shift
  for size in "${SIZES[@]}"; do
    local png="$OUT/${name}_${size}.png"
    env DB_PRO_CAPTURE_TO="$png" DB_PRO_WINDOW_SIZE="$size" "$@" \
      "$BIN" >>"$LOG" 2>&1
    if [ -s "$png" ]; then
      echo "OK   $png"
    else
      echo "FAIL $png"
    fi
  done
}

# --- shell / welcome ---
cap welcome
cap welcome-light               DB_PRO_CAPTURE_WELCOME=1 DB_PRO_CAPTURE_WELCOME_LIGHT=1
cap loading                     DB_PRO_CAPTURE_LOADING=1

# --- connection dialogs ---
cap new-connection              DB_PRO_CAPTURE_NEW_CONNECTION=1
cap new-connection-light        DB_PRO_CAPTURE_NEW_CONNECTION=1 DB_PRO_CAPTURE_CONNECTION_LIGHT=1
cap connection-error            DB_PRO_CAPTURE_CONNECTION_ERROR=1
cap connection-test-pending     DB_PRO_CAPTURE_CONNECTION_TEST_PENDING=1
cap edit-connection             DB_PRO_CAPTURE_EDIT_CONNECTION=1

# --- query workspace ---
cap query                       DB_PRO_CAPTURE_QUERY=1
cap query-light                 DB_PRO_CAPTURE_QUERY=1 DB_PRO_CAPTURE_QUERY_LIGHT=1
cap query-row-limit             DB_PRO_CAPTURE_QUERY=1 DB_PRO_CAPTURE_QUERY_LIMIT=1
cap results                     DB_PRO_CAPTURE_RESULTS=1
cap results-light               DB_PRO_CAPTURE_RESULTS=1 DB_PRO_CAPTURE_RESULTS_LIGHT=1

# --- table workspace ---
cap table                       DB_PRO_CAPTURE_TABLE=1
cap table-light                 DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_LIGHT=1
cap table-ddl                   DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_DDL=1
cap table-structure             DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_STRUCTURE=1
cap table-indexes               DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_INDEXES=1
cap table-profile               DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_PROFILE=1
cap table-foreign-keys          DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_FOREIGN_KEYS=1
cap table-constraints           DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_CONSTRAINTS=1
cap table-dependencies          DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_DEPENDENCIES=1
cap table-completion            DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_COMPLETION=1
cap table-record                DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_RECORD=open

# --- workspaces ---
cap diagram                     DB_PRO_CAPTURE_DIAGRAM=1
cap settings                    DB_PRO_CAPTURE_SETTINGS=1
cap agent                       DB_PRO_CAPTURE_AGENT=1
cap gallery                     DB_PRO_CAPTURE_COMPONENT_GALLERY=1
cap explorer-filter             DB_PRO_CAPTURE_FILTER=1
cap history                     DB_PRO_CAPTURE_HISTORY=1
cap compare                     DB_PRO_CAPTURE_COMPARE=1
cap quick-open                  DB_PRO_CAPTURE_QUICK_OPEN=1
cap quick-open-light            DB_PRO_CAPTURE_QUICK_OPEN=1 DB_PRO_CAPTURE_QUICK_OPEN_LIGHT=1
cap quick-open-empty            DB_PRO_CAPTURE_QUICK_OPEN=1 DB_PRO_CAPTURE_QUICK_OPEN_EMPTY=1

echo "done — $(ls "$OUT"/*.png 2>/dev/null | wc -l | tr -d ' ') pngs"
