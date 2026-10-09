#!/usr/bin/env bash
# ui-audit-matrix.sh — run the headless UI audit (P6) across scenarios and
# aggregate a coverage-matrix JSON.
#
# Usage:
#   bash tools/ui-audit-matrix.sh [binary] [outdir]
#
# Defaults: binary=/data/cargo-target/debug/db-pro-native (must be built with
# `--features capture`), outdir=/tmp/ui-audit-matrix.
#
# Each scenario maps to existing DB_PRO_CAPTURE_* surface/state envs — no new
# test framework; unsupported scenarios are recorded, not hidden.
set -uo pipefail

BIN="${1:-/data/cargo-target/debug/db-pro-native}"
OUT="${2:-/tmp/ui-audit-matrix}"
mkdir -p "$OUT"

if [ ! -x "$BIN" ]; then
    echo "binary not found: $BIN" >&2
    exit 2
fi

# scenario name | viewport | extra env assignments (space-separated K=V)
SCENARIOS=(
    "shell-1440|1440x900|"
    "shell-1920|1920x1080|"
    "shell-800|800x600|"
    "welcome-light|1440x900|DB_PRO_CAPTURE_WELCOME=1 DB_PRO_CAPTURE_WELCOME_LIGHT=1"
    "query-dark|1440x900|DB_PRO_CAPTURE_QUERY=1"
    "query-light|1440x900|DB_PRO_CAPTURE_QUERY=1 DB_PRO_CAPTURE_QUERY_LIGHT=1"
    "table-indexes|1440x900|DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_INDEXES=1"
    "table-structure-light|1440x900|DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_STRUCTURE=1 DB_PRO_CAPTURE_TABLE_LIGHT=1"
    "settings|1440x900|DB_PRO_CAPTURE_SETTINGS=1"
    "explorer-filter|1440x900|DB_PRO_CAPTURE_EXPLORER_FILTER=1"
    "new-connection-dialog|1440x900|DB_PRO_CAPTURE_NEW_CONNECTION=1"
    "quick-open-light|1440x900|DB_PRO_CAPTURE_QUICK_OPEN=1 DB_PRO_CAPTURE_QUICK_OPEN_LIGHT=1"
    "fixture-broken|1440x900|DB_PRO_INSPECTOR_AUDIT_FIXTURE=1"
    "results-dock|1440x900|DB_PRO_CAPTURE_RESULTS=1 DB_PRO_AUDIT_READY=160"
    "results-dock-light|1440x900|DB_PRO_CAPTURE_RESULTS=1 DB_PRO_CAPTURE_RESULTS_LIGHT=1 DB_PRO_AUDIT_READY=160"
    "table-data|1440x900|DB_PRO_CAPTURE_TABLE=1 DB_PRO_AUDIT_READY=180"
    "table-ddl|1440x900|DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_DDL=1 DB_PRO_AUDIT_READY=150"
    "table-profile|1440x900|DB_PRO_CAPTURE_TABLE=1 DB_PRO_CAPTURE_TABLE_PROFILE=1 DB_PRO_AUDIT_READY=150"
    "diagram|1440x900|DB_PRO_CAPTURE_DIAGRAM=1 DB_PRO_AUDIT_READY=140"
    "history|1440x900|DB_PRO_CAPTURE_HISTORY=1 DB_PRO_AUDIT_READY=140"
    "schema-compare|1440x900|DB_PRO_CAPTURE_COMPARE=1 DB_PRO_AUDIT_READY=140"
    "agent|1440x900|DB_PRO_CAPTURE_AGENT=1 DB_PRO_AUDIT_READY=140"
    "component-gallery|1440x900|DB_PRO_CAPTURE_COMPONENT_GALLERY=1 DB_PRO_AUDIT_READY=140"
    "screen-fixture-good|1440x900|DB_PRO_AUDIT_SCREEN_FIXTURE=good DB_PRO_AUDIT_READY=20"
    "screen-fixture-ugly|1440x900|DB_PRO_AUDIT_SCREEN_FIXTURE=ugly DB_PRO_AUDIT_READY=20"
)

RESULTS="$OUT/results.tsv"
: >"$RESULTS"

for row in "${SCENARIOS[@]}"; do
    IFS='|' read -r name viewport envs <<<"$row"
    json="$OUT/$name.json"
    log="$OUT/$name.log"
    # Wayland sessions intermittently fail to map the capture window under
    # load (mutter backpressure) — X11 via Xwayland is deterministic for CI.
    env -i HOME="$HOME" PATH="$PATH" DISPLAY="${DISPLAY:-}" WINIT_UNIX_BACKEND=x11 XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}" \
        XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-}" \
        DB_PRO_AUDIT_JSON="$json" DB_PRO_AUDIT_SCENARIO="$name" DB_PRO_WINDOW_SIZE="$viewport" \
        $envs timeout 120 "$BIN" >"$log" 2>&1
    code=$?
    # 124 = timeout (tool-level failure); 2 = tool error; 1 = audit findings;
    # 0 = clean. JSON absent means the run never produced a report.
    if [ ! -s "$json" ]; then
        status="unsupported"
    elif [ "$code" -eq 0 ]; then
        status="pass"
    elif [ "$code" -eq 1 ]; then
        status="fail"
    else
        status="tool_error"
    fi
    printf '%s\t%s\t%s\t%s\n' "$name" "$viewport" "$status" "$code" >>"$RESULTS"
    echo "[$name] exit=$code status=$status"
done

python3 - "$RESULTS" "$OUT" <<'PY'
import json, sys, pathlib
results, out = sys.argv[1], pathlib.Path(sys.argv[2])
rows = []
for line in open(results):
    name, viewport, status, code = line.rstrip("\n").split("\t")
    entry = {"scenario": name, "viewport": viewport, "status": status, "exit_code": int(code)}
    f = out / f"{name}.json"
    if f.exists():
        d = json.loads(f.read_text())
        s = d.get("summary", {})
        entry.update({
            "schema_version": d.get("schema_version"),
            "theme": d.get("meta", {}).get("theme"),
            "issues": s.get("issues"),
            "errors": s.get("errors"),
            "warnings": s.get("warnings"),
            "verdicts": s.get("verdicts"),
            "rules": {r["id"]: r["verdict"] for r in d.get("rules", [])},
        })
    else:
        entry["note"] = "no report produced — scenario unsupported in this build/run"
    rows.append(entry)
matrix = {"tool": "db-pro-ui-audit", "matrix_version": 1, "scenarios": rows}
(out / "coverage-matrix.json").write_text(json.dumps(matrix, indent=1))
print(f"wrote {out/'coverage-matrix.json'} ({len(rows)} scenarios)")
PY
