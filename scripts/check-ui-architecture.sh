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
connection
feedback
gallery_state
initial_frames_count
overlay
palette
preferences
management
query
saved_tasks
schema
table
task_bridge
theme
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

if rg -n 'table_data|row_reload' "$repo_root/crates/ui/src/table_state.rs"; then
  echo "UI architecture check failed: TableState must not own table data-query lifecycle." >&2
  exit 1
fi
if [[ ! -f "$repo_root/crates/ui/src/table_data_query_state.rs" ]]; then
  echo "UI architecture check failed: missing TableDataQueryState boundary." >&2
  exit 1
fi

explicit_state_modules=(
  "$repo_root/crates/ui/src/activity_bar_view.rs"
  "$repo_root/crates/ui/src/workspace_session.rs"
  "$repo_root/crates/ui/src/connection/view.rs"
  "$repo_root/crates/ui/src/connection/form_fields.rs"
  "$repo_root/crates/ui/src/connection/advanced_panels.rs"
  "$repo_root/crates/ui/src/connection_status.rs"
  "$repo_root/crates/ui/src/connection_events.rs"
  "$repo_root/crates/ui/src/table_editor_context.rs"
  "$repo_root/crates/ui/src/table_data_state.rs"
  "$repo_root/crates/ui/src/table_data_query_state.rs"
  "$repo_root/crates/ui/src/table_editing_state.rs"
  "$repo_root/crates/ui/src/table_editor_values.rs"
  "$repo_root/crates/ui/src/visual_query_builder_state.rs"
  "$repo_root/crates/ui/src/agent_context.rs"
  "$repo_root/crates/ui/src/synthetic_data.rs"
  "$repo_root/crates/ui/src/masking.rs"
  "$repo_root/crates/ui/src/security_rls.rs"
  "$repo_root/crates/ui/src/monitoring_state.rs"
  "$repo_root/crates/ui/src/audit_state.rs"
  "$repo_root/crates/ui/src/routine_state.rs"
  "$repo_root/crates/ui/src/transfer_state.rs"
  "$repo_root/crates/ui/src/synthetic_data_state.rs"
  "$repo_root/crates/ui/src/masking_state.rs"
  "$repo_root/crates/ui/src/pg_settings_state.rs"
  "$repo_root/crates/ui/src/fdw_state.rs"
  "$repo_root/crates/ui/src/replication_state.rs"
  "$repo_root/crates/ui/src/event_trigger_state.rs"
  "$repo_root/crates/ui/src/security_state.rs"
  "$repo_root/crates/ui/src/database_management_state.rs"
  "$repo_root/crates/ui/src/schema_workspace_state.rs"
  "$repo_root/crates/ui/src/diagram_view.rs"
  "$repo_root/crates/ui/src/diagram_canvas_view.rs"
  "$repo_root/crates/ui/src/diagram_design_actions.rs"
  "$repo_root/crates/ui/src/diagram_design_panel_view.rs"
  "$repo_root/crates/ui/src/palette_catalog.rs"
  "$repo_root/crates/ui/src/query_snippets.rs"
  "$repo_root/crates/ui/src/query_diagnostics_view.rs"
  "$repo_root/crates/ui/src/navigation_view.rs"
  "$repo_root/crates/ui/src/maintenance_activity_view.rs"
  "$repo_root/crates/ui/src/result_grid_export.rs"
  "$repo_root/crates/ui/src/result_grid_selection.rs"
  "$repo_root/crates/ui/src/schema_compare_view.rs"
  "$repo_root/crates/ui/src/schema_workbench_form.rs"
  "$repo_root/crates/ui/src/schema_workbench_secondary_view.rs"
  "$repo_root/crates/ui/src/query_search_view.rs"
  "$repo_root/crates/ui/src/query_context_view.rs"
  "$repo_root/crates/ui/src/query_context_picker_view.rs"
  "$repo_root/crates/ui/src/query_parameters_view.rs"
  "$repo_root/crates/ui/src/query_run_control_view.rs"
  "$repo_root/crates/ui/src/query_output_tabs_view.rs"
  "$repo_root/crates/ui/src/query_output_panes_view.rs"
  "$repo_root/crates/ui/src/query_output_actions_view.rs"
  "$repo_root/crates/ui/src/query_results_pane_view.rs"
  "$repo_root/crates/ui/src/result_grid_toolbar_view.rs"
  "$repo_root/crates/ui/src/result_grid_projection.rs"
  "$repo_root/crates/ui/src/table_data_placeholder_view.rs"
  "$repo_root/crates/ui/src/table_data_pagination_view.rs"
  "$repo_root/crates/ui/src/table_data_filter_view.rs"
)
for module in "${explicit_state_modules[@]}"; do
  if rg -n '^impl DbProApp|\bDbProApp\b' "$module"; then
    echo "UI architecture check failed: explicit-state feature helpers must depend on state/context, not DbProApp." >&2
    exit 1
  fi
done
for reducer in \
  "$repo_root/crates/ui/src/agent_events.rs" \
  "$repo_root/crates/ui/src/connection_events.rs" \
  "$repo_root/crates/ui/src/ddl_events.rs" \
  "$repo_root/crates/ui/src/file_picker_events.rs" \
  "$repo_root/crates/ui/src/management_events.rs" \
  "$repo_root/crates/ui/src/schema_events.rs" \
  "$repo_root/crates/ui/src/table_events.rs" \
  "$repo_root/crates/ui/src/query_library_events.rs" \
  "$repo_root/crates/ui/src/query_prediction_events.rs" \
  "$repo_root/crates/ui/src/query_save_events.rs" \
  "$repo_root/crates/ui/src/query_history_events.rs" \
  "$repo_root/crates/ui/src/query_queue_events.rs" \
  "$repo_root/crates/ui/src/query_execution_events.rs" \
  "$repo_root/crates/ui/src/query_result_events.rs" \
  "$repo_root/crates/ui/src/query_multi_result_events.rs" \
  "$repo_root/crates/ui/src/query_failure_events.rs"; do
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

for module in event_router agent_events connection_events ddl_events file_picker_events management_events operation_events schema_events table_events query_library_events query_prediction_events query_save_events query_history_events query_queue_events query_execution_events query_result_events query_multi_result_events; do
  test -f "$repo_root/crates/ui/src/${module}.rs" || {
    echo "UI architecture check failed: missing event module ${module}.rs." >&2
    exit 1
  }
done

echo "UI architecture boundary: PASS"
