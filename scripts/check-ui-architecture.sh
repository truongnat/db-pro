#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app_file="$repo_root/crates/ui/src/app.rs"
events_file="$repo_root/crates/ui/src/events.rs"

# DbProApp is deliberately an allowlisted composition root. A new field must
# be a feature aggregate, an adapter, or shell composition state; otherwise it
# belongs in the owning feature state module.
expected_fields=$(cat <<'EOF'
agent
audit
connection
diagram
event_trigger
fdw
feedback
gallery_state
initial_frames_count
overlay
palette
preferences
masking
monitoring
pg_settings
replication
routine
security
query_editor
query_execution
query_library
query_output_state
query_session_state
saved_tasks
schema_explorer
schema_workbench
schema_compare
synthetic_data
table_data
table_mutation
table_state
task_bridge
theme
transfer
welcome
workspace
EOF
)

actual_fields=$(awk '
  /^pub struct DbProApp \{/ { inside=1; next }
  inside && /^}/ { exit }
  inside { print }
' "$app_file" | grep -E '^[[:space:]]*(pub\([^)]*\)[[:space:]]+|pub[[:space:]]+)?[A-Za-z_][A-Za-z0-9_]*:' | sed -E 's/^[[:space:]]*(pub\([^)]*\)[[:space:]]+|pub[[:space:]]+)?([A-Za-z_][A-Za-z0-9_]*):.*/\2/' | sed '/^$/d' | sort)

if ! diff -u <(printf '%s\n' "$expected_fields" | sed '/^$/d' | sort) <(printf '%s\n' "$actual_fields"); then
  echo "UI architecture check failed: DbProApp fields changed outside the allowlist." >&2
  exit 1
fi

if rg -n 'fn on_[A-Za-z0-9_]+\(' "$events_file"; then
  echo "UI architecture check failed: feature event handlers leaked back into events.rs." >&2
  exit 1
fi

router_file="$repo_root/crates/ui/src/event_router.rs"
if rg --pcre2 -n 'self\.(?!on_[A-Za-z0-9_]+\(|apply_runtime_event\()' "$router_file"; then
  echo "UI architecture check failed: event_router.rs contains direct state/orchestration access." >&2
  exit 1
fi

if ! rg -q 'drain_events\(crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME\)' "$repo_root/crates/ui/src/events.rs"; then
  echo "UI architecture check failed: runtime events are not drained with the per-frame bound." >&2
  exit 1
fi

direct_sends=$(rg -n 'task_bridge\.send\(' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app\.rs:' || true)
if [[ -n "$direct_sends" ]]; then
  echo "$direct_sends" >&2
  echo "UI architecture check failed: feature code bypasses the command dispatch adapter." >&2
  exit 1
fi

if [[ -e "$repo_root/crates/ui/src/database_operations_state.rs" ]]; then
  echo "UI architecture check failed: database_operations_state.rs catch-all must stay deleted." >&2
  exit 1
fi

connection_renderers=(
  "$repo_root/crates/ui/src/connection/view.rs"
  "$repo_root/crates/ui/src/connection/form_fields.rs"
  "$repo_root/crates/ui/src/connection/advanced_panels.rs"
  "$repo_root/crates/ui/src/connection_status.rs"
  "$repo_root/crates/ui/src/connection_events.rs"
)
for renderer in "${connection_renderers[@]}"; do
  if rg -n '^impl DbProApp|\bDbProApp\b' "$renderer"; then
    echo "UI architecture check failed: connection feature helpers must depend on explicit state/context, not DbProApp." >&2
    exit 1
  fi
done
for reducer in \
  "$repo_root/crates/ui/src/connection_events.rs" \
  "$repo_root/crates/ui/src/schema_events.rs"; do
  if rg -n '^impl DbProApp|\bDbProApp\b' "$reducer"; then
    echo "UI architecture check failed: feature event reducers must depend on explicit state, not DbProApp." >&2
    exit 1
  fi
done
if rg -n 'database_operations|DatabaseOperationsState' "$repo_root/crates/ui/src" --glob '*.rs'; then
  echo "UI architecture check failed: database-management state must use feature-owned aggregates." >&2
  exit 1
fi

state_field_leaks=$(rg -n '^\s*pub(\(crate\))? [A-Za-z_][A-Za-z0-9_]*:' \
  "$repo_root/crates/ui/src"/*_state.rs \
  "$repo_root/crates/ui/src/database_feature_states.rs" \
  "$repo_root/crates/ui/src/schema_workbench.rs" \
  "$repo_root/crates/ui/src/connection/state.rs" \
  "$repo_root/crates/ui/src/connection/lifecycle.rs" \
  "$repo_root/crates/ui/src/connection/catalog.rs" || true)
if [[ -n "$state_field_leaks" ]]; then
  echo "$state_field_leaks" >&2
  echo "UI architecture check failed: feature state fields must stay inside the app boundary." >&2
  exit 1
fi

connection_dialog_boundary_leaks=$(rg -n 'pub\(in crate::app\)' \
  "$repo_root/crates/ui/src/connection/state.rs" || true)
if [[ -n "$connection_dialog_boundary_leaks" ]]; then
  echo "$connection_dialog_boundary_leaks" >&2
  echo "UI architecture check failed: connection dialog state must not expose app-wide fields." >&2
  exit 1
fi

for module in event_router agent_events connection_events operation_events schema_events table_events; do
  test -f "$repo_root/crates/ui/src/${module}.rs" || {
    echo "UI architecture check failed: missing event module ${module}.rs." >&2
    exit 1
  }
done

echo "UI architecture boundary: PASS"
