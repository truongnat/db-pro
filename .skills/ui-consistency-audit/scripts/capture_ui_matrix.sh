#!/usr/bin/env bash
# Repeatable UI capture matrix for consistency audits.
#
# Usage:
#   bash .skills/ui-consistency-audit/scripts/capture_ui_matrix.sh --surface query [--audit] [--light] [--size 1440x900]
#   bash .skills/ui-consistency-audit/scripts/capture_ui_matrix.sh --all [--audit]
#   bash .skills/ui-consistency-audit/scripts/capture_ui_matrix.sh --surface query --variant limit
#
# Requires a display. Under a headless host, wrap the command with:
#   xvfb-run -a -s "-screen 0 1920x1080x24" bash .skills/.../capture_ui_matrix.sh ...
# The app prefers Wayland when WAYLAND_DISPLAY is inherited; this script forces X11.
set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
# Honor a workspace-level CARGO_TARGET_DIR / build config: cargo may place the
# binary outside <repo>/target (this workspace resolves it via `cargo metadata`).
if [[ -z "${DB_PRO_CAPTURE_BIN:-}" ]]; then
    target_dir="$(cd "$repo_root" && cargo metadata --format-version 1 --no-deps 2>/dev/null \
        | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])' 2>/dev/null)"
    BIN="${target_dir:-$repo_root/target}/debug/db-pro-native"
else
    BIN="$DB_PRO_CAPTURE_BIN"
fi
OUT="${DB_PRO_CAPTURE_OUT:-/tmp/ui-audit/round}"
SIZE="1440x900"
SURFACE=""
VARIANT=""
AUDIT=0
LIGHT=0
ALL=0
SETTLE="${DB_PRO_CAPTURE_SETTLE_FRAMES:-180}"

usage() {
    sed -n '2,18p' "$0"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --surface) SURFACE="${2:-}"; shift 2 ;;
        --variant) VARIANT="${2:-}"; shift 2 ;;
        --size) SIZE="${2:-}"; shift 2 ;;
        --audit) AUDIT=1; shift ;;
        --light) LIGHT=1; shift ;;
        --all) ALL=1; shift ;;
        -h|--help) usage ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [[ ! -x "$BIN" ]]; then
    echo "capture binary not found: $BIN" >&2
    echo "build it with: cargo build -p db-pro-native --features capture" >&2
    exit 1
fi

mkdir -p "$OUT/png" "$OUT/audit"
RESULTS="$OUT/capture.tsv"
[[ -f "$RESULTS" ]] || printf 'surface\tviewport\tstatus\texit\n' > "$RESULTS"

# surface | env flags (without theme/size)
surfaces() {
    cat <<'EOF'
welcome|DB_PRO_CAPTURE_WELCOME=1
query|DB_PRO_CAPTURE_QUERY=1
results|DB_PRO_CAPTURE_RESULTS=1
table|DB_PRO_CAPTURE_TABLE=1
new-connection|DB_PRO_CAPTURE_NEW_CONNECTION=1
connection-error|DB_PRO_CAPTURE_CONNECTION_ERROR=1
edit-connection|DB_PRO_CAPTURE_EDIT_CONNECTION=1
settings|DB_PRO_CAPTURE_SETTINGS=1
quick-open|DB_PRO_CAPTURE_QUICK_OPEN=1
explorer-filter|DB_PRO_CAPTURE_FILTER=1
diagram|DB_PRO_CAPTURE_DIAGRAM=1
history|DB_PRO_CAPTURE_HISTORY=1
compare|DB_PRO_CAPTURE_COMPARE=1
agent|DB_PRO_CAPTURE_AGENT=1
gallery|DB_PRO_CAPTURE_COMPONENT_GALLERY=1
files|DB_PRO_CAPTURE_FILES=1
tools|DB_PRO_CAPTURE_TOOLS=1
EOF
}

variant_env() {
    case "$1" in
        limit) echo "DB_PRO_CAPTURE_QUERY_LIMIT=1" ;;
        thread) echo "DB_PRO_CAPTURE_AGENT_THREAD=1" ;;
        empty) echo "DB_PRO_CAPTURE_QUICK_OPEN_EMPTY=1" ;;
        *) echo "" ;;
    esac
}

run_one() {
    local name="$1" envs="$2" dark="$3"
    local png="$OUT/png/${name}-${SIZE}-${dark}.png"
    local log="$OUT/audit/${name}-${SIZE}-${dark}.log"
    local audit_flag=""
    if [[ "$AUDIT" -eq 1 ]]; then
        audit_flag="DB_PRO_AUDIT_JSON=$OUT/audit/${name}-${SIZE}-${dark}.json DB_PRO_INSPECTOR_DUMP=1"
    fi
    # shellcheck disable=SC2086
    env -i HOME="$HOME" PATH="$PATH" \
        DISPLAY="${DISPLAY:-:0}" WINIT_UNIX_BACKEND=x11 \
        XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}" \
        XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
        DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-}" \
        DB_PRO_CAPTURE_TO="$png" DB_PRO_WINDOW_SIZE="$SIZE" \
        DB_PRO_CAPTURE_SETTLE_FRAMES="$SETTLE" $audit_flag $envs \
        timeout 120 "$BIN" >"$log" 2>&1
    local code=$?
    local status=missing
    [[ -s "$png" ]] && status=captured
    printf '%s\t%s\t%s\t%s\n' "$name" "$SIZE" "$status" "$code" >> "$RESULTS"
    echo "[$name $SIZE $dark] exit=$code $status"
}

light_env() {
    case "$1" in
        query) echo "DB_PRO_CAPTURE_QUERY_LIGHT=1" ;;
        results) echo "DB_PRO_CAPTURE_RESULTS_LIGHT=1" ;;
        quick-open) echo "DB_PRO_CAPTURE_QUICK_OPEN_LIGHT=1" ;;
        new-connection|connection-error|edit-connection) echo "DB_PRO_CAPTURE_CONNECTION_LIGHT=1" ;;
        welcome) echo "DB_PRO_CAPTURE_WELCOME_LIGHT=1" ;;
        files) echo "DB_PRO_CAPTURE_FILES_LIGHT=1" ;;
        tools) echo "DB_PRO_CAPTURE_TOOLS_LIGHT=1" ;;
        *) echo "" ;;
    esac
}

if [[ "$ALL" -eq 1 ]]; then
    while IFS='|' read -r name envs; do
        theme="dark"
        if [[ "$LIGHT" -eq 1 ]]; then
            theme="light"
            envs="$envs $(light_env "$name")"
        fi
        run_one "$name" "$envs" "$theme"
    done < <(surfaces)
    exit 0
fi

if [[ -z "$SURFACE" ]]; then
    echo "pass --surface <name> or --all (see --help)" >&2
    exit 2
fi

line="$(surfaces | awk -F'|' -v s="$SURFACE" '$1==s {print $2}')"
if [[ -z "$line" ]]; then
    echo "unknown surface: $SURFACE" >&2
    exit 2
fi

extra="$(variant_env "$VARIANT")"
theme="dark"
if [[ "$LIGHT" -eq 1 ]]; then
    theme="light"
    # Most fixtures use DB_PRO_CAPTURE_<UPPER>_LIGHT; query/files/tools/results/quick-open follow it.
    case "$SURFACE" in
        query) line="$line DB_PRO_CAPTURE_QUERY_LIGHT=1" ;;
        results) line="$line DB_PRO_CAPTURE_RESULTS_LIGHT=1" ;;
        quick-open) line="$line DB_PRO_CAPTURE_QUICK_OPEN_LIGHT=1" ;;
        new-connection|connection-error|edit-connection) line="$line DB_PRO_CAPTURE_CONNECTION_LIGHT=1" ;;
        welcome) line="$line DB_PRO_CAPTURE_WELCOME_LIGHT=1" ;;
        files) line="$line DB_PRO_CAPTURE_FILES_LIGHT=1" ;;
        tools) line="$line DB_PRO_CAPTURE_TOOLS_LIGHT=1" ;;
    esac
fi

run_one "$SURFACE" "$line $extra" "$theme"
