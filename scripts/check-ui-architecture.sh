#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app_file="$repo_root/crates/app/src/app/mod.rs"
events_file="$repo_root/crates/app/src/runtime/events.rs"

if ! rg -q '^include!\("app_modules\.rs"\);$' "$app_file"; then
  echo "UI architecture check failed: app/mod.rs must include the isolated module topology registry." >&2
  exit 1
fi
if rg -n '^#\[path = ' "$app_file"; then
  echo "UI architecture check failed: app/mod.rs must not own feature module path declarations." >&2
  exit 1
fi
if rg -n 'UiCommand::' "$app_file"; then
  echo "UI architecture check failed: app/mod.rs must delegate runtime command construction to feature adapters." >&2
  exit 1
fi

if rg -n 'egui::(CentralPanel|SidePanel|TopBottomPanel|Window::new)|\.show\(.*\|ui\|' "$app_file"; then
  echo "UI architecture check failed: app/mod.rs must remain a composition root without egui painting." >&2
  exit 1
fi

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
table_copy
table
task_bridge
theme
dev_tools
capture_evidence_seeded
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

router_file="$repo_root/crates/app/src/runtime/event_router.rs"
if rg --pcre2 -n 'self\.(?!on_[A-Za-z0-9_]+\(|apply_runtime_event\()' "$router_file"; then
  echo "UI architecture check failed: event_router.rs contains direct state/orchestration access." >&2
  exit 1
fi

if ! rg -q 'drain_events\(crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME\)' "$repo_root/crates/app/src/runtime/events.rs"; then
  echo "UI architecture check failed: runtime events are not drained with the per-frame bound." >&2
  exit 1
fi

direct_sends=$(rg --pcre2 -U -n 'task_bridge\s*(?:\n\s*)?\.send\(' "$repo_root/crates/app/src" --glob '*.rs' | rg -v '/app/mod\.rs:' || true)
if [[ -n "$direct_sends" ]]; then
  echo "$direct_sends" >&2
  echo "UI architecture check failed: feature code bypasses the command dispatch adapter." >&2
  exit 1
fi

direct_best_effort_sends=$(rg --pcre2 -U -n 'task_bridge\s*(?:\n\s*)?\.send_best_effort\(' "$repo_root/crates/app/src" --glob '*.rs' | rg -v '/app/mod\.rs:' || true)
if [[ -n "$direct_best_effort_sends" ]]; then
  echo "$direct_best_effort_sends" >&2
  echo "UI architecture check failed: feature code bypasses the command dispatch adapter with best-effort sends." >&2
  exit 1
fi

direct_request_allocations=$(rg -n 'task_bridge\.next_request_id\(\)' "$repo_root/crates/app/src" --glob '*.rs' | rg -v '/app/mod\.rs:' || true)
if [[ -n "$direct_request_allocations" ]]; then
  echo "$direct_request_allocations" >&2
  echo "UI architecture check failed: feature code must allocate request IDs through the composition-root port." >&2
  exit 1
fi

direct_windows=$(rg -n 'egui::Window::new' "$repo_root/crates/app/src" --glob '*.rs' || true)
if [[ -n "$direct_windows" ]]; then
  echo "$direct_windows" >&2
  echo "UI architecture check failed: feature dialogs must use components::dialog::Dialog." >&2
  exit 1
fi

saved_tasks_file="$repo_root/crates/app/src/tools/tasks_view.rs"
if rg -n 'SELECT \* FROM \{|VACUUM \{|ANALYZE \{' "$saved_tasks_file"; then
  echo "UI architecture check failed: saved task SQL must be prepared by the provider-aware SQL boundary." >&2
  exit 1
fi
if ! rg -q 'build_export_query|build_maintenance_query' "$saved_tasks_file"; then
  echo "UI architecture check failed: saved task dispatch must use the provider-aware SQL boundary." >&2
  exit 1
fi

if [[ -e "$repo_root/crates/app/src/database_operations_state.rs" ]]; then
  echo "UI architecture check failed: database_operations_state.rs catch-all must stay deleted." >&2
  exit 1
fi
if [[ -e "$repo_root/crates/app/src/schema/database_operations_state.rs" ]]; then
  echo "UI architecture check failed: schema/database_operations_state.rs catch-all must stay deleted." >&2
  exit 1
fi

if rg -n 'table_data|row_reload' "$repo_root/crates/app/src/table/table_state.rs"; then
  echo "UI architecture check failed: TableState must not own table data-query lifecycle." >&2
  exit 1
fi

if rg -n '&mut WorkspaceFeatureState|WorkspaceFeatureState' "$repo_root/crates/app/src/shell/activity_bar_view.rs"; then
  echo "UI architecture check failed: activity_bar_view.rs must emit intents instead of mutating workspace state." >&2
  exit 1
fi
if [[ ! -f "$repo_root/crates/app/src/table/table_data_query_state.rs" ]]; then
  echo "UI architecture check failed: missing TableDataQueryState boundary." >&2
  exit 1
fi

explicit_state_modules=(
  "$repo_root/crates/app/src/shell/activity_bar_view.rs"
  "$repo_root/crates/app/src/workspace/workspace_session.rs"
  "$repo_root/crates/app/src/connection/view.rs"
  "$repo_root/crates/app/src/connection/form_fields.rs"
  "$repo_root/crates/app/src/connection/advanced_panels.rs"
  "$repo_root/crates/app/src/connection/connection_status.rs"
  "$repo_root/crates/app/src/connection/connection_events.rs"
  "$repo_root/crates/app/src/table/table_editor_context.rs"
  "$repo_root/crates/app/src/table/table_data_state.rs"
  "$repo_root/crates/app/src/table/table_data_surface_view.rs"
  "$repo_root/crates/app/src/table/table_data_query_state.rs"
  "$repo_root/crates/app/src/table/table_editing_state.rs"
  "$repo_root/crates/app/src/table/table_editor_values.rs"
  "$repo_root/crates/app/src/visual_query_builder/visual_query_builder_state.rs"
  "$repo_root/crates/app/src/visual_query_builder/visual_query_builder_surface_view.rs"
  "$repo_root/crates/app/src/agent/agent_context.rs"
  "$repo_root/crates/app/src/agent/agent_confirmation.rs"
  "$repo_root/crates/app/src/agent/agent_context_actions_view.rs"
  "$repo_root/crates/app/src/agent/agent_header_view.rs"
  "$repo_root/crates/app/src/agent/agent_surface_view.rs"
  "$repo_root/crates/app/src/agent/agent_thread_surface_view.rs"
  "$repo_root/crates/app/src/tools/audit_surface_view.rs"
  "$repo_root/crates/app/src/tools/event_trigger_surface_view.rs"
  "$repo_root/crates/app/src/tools/fdw_surface_view.rs"
  "$repo_root/crates/app/src/files/files_surface_view.rs"
  "$repo_root/crates/app/src/files/files_secondary_tabs_view.rs"
  "$repo_root/crates/app/src/tools/pg_settings_surface_view.rs"
  "$repo_root/crates/app/src/tools/replication_surface_view.rs"
  "$repo_root/crates/app/src/monitoring/monitoring_surface_view.rs"
  "$repo_root/crates/app/src/agent/agent_patch.rs"
  "$repo_root/crates/app/src/agent/agent_result_projection.rs"
  "$repo_root/crates/app/src/explorer/schema_explorer_state.rs"
  "$repo_root/crates/app/src/agent/agent_settings_view.rs"
  "$repo_root/crates/app/src/settings/settings_appearance_view.rs"
  "$repo_root/crates/app/src/settings/settings_backup_view.rs"
  "$repo_root/crates/app/src/agent/agent_workflow_reducer.rs"
  "$repo_root/crates/app/src/tools/synthetic_data.rs"
  "$repo_root/crates/app/src/tools/masking.rs"
  "$repo_root/crates/app/src/security/security_rls.rs"
  "$repo_root/crates/app/src/monitoring/monitoring_state.rs"
  "$repo_root/crates/app/src/monitoring/monitoring_snapshot_view.rs"
  "$repo_root/crates/app/src/tools/audit_state.rs"
  "$repo_root/crates/app/src/schema/routine_state.rs"
  "$repo_root/crates/app/src/tools/transfer_state.rs"
  "$repo_root/crates/app/src/tools/transfer_harness_view.rs"
  "$repo_root/crates/app/src/tools/transfer_activity_surface_view.rs"
  "$repo_root/crates/app/src/tools/saved_tasks_surface_view.rs"
  "$repo_root/crates/app/src/tools/synthetic_data_state.rs"
  "$repo_root/crates/app/src/tools/masking_state.rs"
  "$repo_root/crates/app/src/tools/pg_settings_state.rs"
  "$repo_root/crates/app/src/tools/fdw_state.rs"
  "$repo_root/crates/app/src/tools/replication_state.rs"
  "$repo_root/crates/app/src/tools/event_trigger_state.rs"
  "$repo_root/crates/app/src/security/security_state.rs"
  "$repo_root/crates/app/src/schema/database_management_state.rs"
  "$repo_root/crates/app/src/schema/schema_workspace_state.rs"
  "$repo_root/crates/app/src/diagram_surface/diagram_view.rs"
  "$repo_root/crates/app/src/diagram_surface/diagram_canvas_view.rs"
  "$repo_root/crates/app/src/diagram_surface/diagram_design_actions.rs"
  "$repo_root/crates/app/src/diagram_surface/diagram_design_panel_view.rs"
  "$repo_root/crates/app/src/palette/palette_catalog.rs"
  "$repo_root/crates/app/src/palette/palette_search_view.rs"
  "$repo_root/crates/app/src/palette/palette_surface_view.rs"
  "$repo_root/crates/app/src/files/files_agent_context_view.rs"
  "$repo_root/crates/app/src/files/files_tree_view.rs"
  "$repo_root/crates/app/src/files/files_search_view.rs"
  "$repo_root/crates/app/src/files/files_tasks_view.rs"
  "$repo_root/crates/app/src/files/files_git_view.rs"
  "$repo_root/crates/app/src/settings/settings_system_view.rs"
  "$repo_root/crates/app/src/settings/settings_surface_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_data_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_chrome_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_surface_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_queries_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_problems_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_query_library_view.rs"
  "$repo_root/crates/app/src/shell/sidebar/sidebar_query_shortcuts_view.rs"
  "$repo_root/crates/app/src/shell/chrome/shell_frame_view.rs"
  "$repo_root/crates/app/src/shell/chrome/shell_topbar_view.rs"
  "$repo_root/crates/app/src/shell/chrome/shell_statusbar_view.rs"
  "$repo_root/crates/app/src/shell/welcome_surface_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_toolbar_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_connection_row_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_connection_node_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_database_node_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_schema_feedback_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_dialog_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_save_dialog_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_transaction_surface_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_schema_node_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_schema_object_row_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_schema_object_folders_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_schema_objects_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_schema_tree_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_surface_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_navigation.rs"
  "$repo_root/crates/app/src/explorer/explorer_table_row_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_table_details_view.rs"
  "$repo_root/crates/app/src/explorer/explorer_table_folder_view.rs"
  "$repo_root/crates/app/src/table/table_workspace_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_snippets.rs"
  "$repo_root/crates/app/src/query_workbench/query_diagnostics_view.rs"
  "$repo_root/crates/app/src/shell/navigation_view.rs"
  "$repo_root/crates/app/src/tools/maintenance_activity_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_export.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_selection.rs"
  "$repo_root/crates/app/src/schema/schema_compare_view.rs"
  "$repo_root/crates/app/src/schema/schema_object_surface_view.rs"
  "$repo_root/crates/app/src/schema/schema_object_resolver.rs"
  "$repo_root/crates/app/src/schema/schema_workbench_form.rs"
  "$repo_root/crates/app/src/schema/schema_workbench_mutation.rs"
  "$repo_root/crates/app/src/schema/schema_workbench_secondary_view.rs"
  "$repo_root/crates/app/src/schema/schema_workbench_surface_view.rs"
  "$repo_root/crates/app/src/security/security_confirmation_view.rs"
  "$repo_root/crates/app/src/security/security_surface_view.rs"
  "$repo_root/crates/app/src/settings/settings_navigation_view.rs"
  "$repo_root/crates/app/src/settings/settings_keybindings_view.rs"
  "$repo_root/crates/app/src/settings/settings_diagnostics_view.rs"
  "$repo_root/crates/app/src/settings/settings_general_view.rs"
  "$repo_root/crates/app/src/settings/settings_editor_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_search_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_shell_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_context_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_context_picker_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_parameters_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_completion_popup_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_editor_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_actions_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_snippets_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_run_control_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_explain_actions.rs"
  "$repo_root/crates/app/src/query_workbench/query_save_actions.rs"
  "$repo_root/crates/app/src/query_workbench/query_output_tabs_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_output_dock_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_output_panes_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_output_actions_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_layout_surface_view.rs"
  "$repo_root/crates/app/src/query_workbench/query_results_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_toolbar_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_body_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_header_menu_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_header_content_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_header_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_interaction_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_keyboard_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_row_gutter_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_row_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_cell_menu_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_cell_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_cell_editor_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_inspector_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_record_surface_view.rs"
  "$repo_root/crates/app/src/workspace/workspace_tabs_surface_view.rs"
  "$repo_root/crates/app/src/result_grid/result_grid_projection.rs"
  "$repo_root/crates/app/src/table/table_data_placeholder_view.rs"
  "$repo_root/crates/app/src/table/table_structure_surface_view.rs"
  "$repo_root/crates/app/src/table/table_scroll_surface_view.rs"
  "$repo_root/crates/app/src/table/table_indexes_surface_view.rs"
  "$repo_root/crates/app/src/table/table_relations_surface_view.rs"
  "$repo_root/crates/app/src/table/table_metadata_surface_view.rs"
  "$repo_root/crates/app/src/table/table_profile_surface_view.rs"
  "$repo_root/crates/app/src/table/table_data_pagination_view.rs"
  "$repo_root/crates/app/src/table/table_data_filter_view.rs"
  "$repo_root/crates/app/src/table/table_data_mutation_toolbar_view.rs"
)
for module in "${explicit_state_modules[@]}"; do
  # DbProApp inside #[cfg(test)] blocks is allowed: tests construct the shell.
  if awk '/#\[cfg\(test\)\]/{exit} /^impl DbProApp|\bDbProApp\b/{found=1} END{exit !found}' "$module"; then
    echo "UI architecture check failed: explicit-state feature helpers must depend on state/context, not DbProApp." >&2
    exit 1
  fi
done

# Feature state modules live one level deep inside feature folders after the
# 2026-10-14 reorg — recurse for *_state.rs instead of the old flat src/*_state.rs
# glob, which silently matched nothing. app/app_state.rs is the composition-root
# state and is exempt (it may reference DbProApp).
feature_state_modules=()
while IFS= read -r module; do
  feature_state_modules+=("$module")
done < <(rg --files "$repo_root/crates/app/src" -g '*_state.rs')

for state_module in "${feature_state_modules[@]}"; do
  [[ "$state_module" == "$repo_root/crates/app/src/app/app_state.rs" ]] && continue
  if rg -n '^impl DbProApp|\bDbProApp\b' "$state_module"; then
    echo "UI architecture check failed: feature state modules must not depend on DbProApp." >&2
    exit 1
  fi
done

# UiCommand is runtime protocol: feature state modules consume events, never
# construct commands. app/app_state.rs is exempt — the composition root owns
# command dispatch.
for state_module in "${feature_state_modules[@]}"; do
  [[ "$state_module" == "$repo_root/crates/app/src/app/app_state.rs" ]] && continue
  if rg -n '\bUiCommand\b' "$state_module"; then
    echo "UI architecture check failed: feature state modules must not construct runtime protocol commands." >&2
    exit 1
  fi
done
if rg -n '\bUiCommand\b' "$repo_root/crates/app/src/schema/schema_workbench.rs"; then
  echo "UI architecture check failed: SchemaWorkbenchState must not construct runtime protocol commands." >&2
  exit 1
fi
if rg -n '\bUiCommand\b' "$repo_root/crates/app/src/explorer/explorer_navigation.rs"; then
  echo "UI architecture check failed: explorer navigation state must not construct runtime protocol commands." >&2
  exit 1
fi
if rg -n '\bUiCommand\b' "$repo_root/crates/app/src/query_workbench/query_execution_actions.rs"; then
  echo "UI architecture check failed: query execution state must not depend on runtime protocol commands." >&2
  exit 1
fi
if rg -n '\bUiCommand\b' "$repo_root/crates/app/src/query_workbench/query_save_actions.rs"; then
  echo "UI architecture check failed: query save state must not depend on runtime protocol commands." >&2
  exit 1
fi
if rg -n '\bUiCommand\b' "$repo_root/crates/app/src/connection/lifecycle.rs"; then
  echo "UI architecture check failed: connection lifecycle state must not construct runtime protocol commands." >&2
  exit 1
fi

# Feature event reducers are *_events.rs inside feature folders. src/runtime/
# holds the dispatcher plumbing (operation_events.rs) — it is not a feature
# reducer and is excluded.
feature_event_modules=()
while IFS= read -r module; do
  feature_event_modules+=("$module")
done < <(rg --files "$repo_root/crates/app/src" -g '*_events.rs' | rg -v '/runtime/')

for reducer in "${feature_event_modules[@]}"; do
  if rg -n '^impl DbProApp|\bDbProApp\b' "$reducer"; then
    echo "UI architecture check failed: feature event reducers must depend on explicit state, not DbProApp." >&2
    exit 1
  fi
done
if rg -n 'database_operations|DatabaseOperationsState' "$repo_root/crates/app/src" "$repo_root/crates/ui/src" --glob '*.rs'; then
  echo "UI architecture check failed: database-management state must use feature-owned aggregates." >&2
  exit 1
fi

state_field_leaks=$(rg -n '^\s*pub(\(crate\))? [A-Za-z_][A-Za-z0-9_]*:' \
  "$repo_root/crates/app/src" --glob '*_state.rs' \
  "$repo_root/crates/app/src/schema/schema_workbench.rs" \
  "$repo_root/crates/app/src/connection/state.rs" \
  "$repo_root/crates/app/src/connection/lifecycle.rs" \
  "$repo_root/crates/app/src/connection/catalog.rs" || true)
if [[ -n "$state_field_leaks" ]]; then
  echo "$state_field_leaks" >&2
  echo "UI architecture check failed: feature state fields must stay inside the app boundary." >&2
  exit 1
fi

connection_dialog_boundary_leaks=$(rg -n 'pub\(in crate::app\)' \
  "$repo_root/crates/app/src/connection/state.rs" || true)
if [[ -n "$connection_dialog_boundary_leaks" ]]; then
  echo "$connection_dialog_boundary_leaks" >&2
  echo "UI architecture check failed: connection dialog state must not expose app-wide fields." >&2
  exit 1
fi

for event_module in \
  "$repo_root/crates/app/src/runtime/events.rs" \
  "$repo_root/crates/app/src/runtime/event_router.rs" \
  "$repo_root/crates/app/src/runtime/operation_events.rs" \
  "$repo_root/crates/app/src/agent/agent_events.rs" \
  "$repo_root/crates/app/src/connection/connection_events.rs" \
  "$repo_root/crates/app/src/files/file_picker_events.rs" \
  "$repo_root/crates/app/src/tools/management_events.rs" \
  "$repo_root/crates/app/src/schema/schema_events.rs" \
  "$repo_root/crates/app/src/table/ddl_events.rs" \
  "$repo_root/crates/app/src/table/table_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_library_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_prediction_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_save_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_history_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_queue_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_execution_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_result_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_multi_result_events.rs" \
  "$repo_root/crates/app/src/query_workbench/query_failure_events.rs"; do
  test -f "$event_module" || {
    echo "UI architecture check failed: missing event module ${event_module#$repo_root/crates/ui/src/}." >&2
    exit 1
  }
done

# RULES.md: view code must use crates/ui/src/components primitives, never raw
# egui widgets. Exemptions: components/ (the wrappers themselves), dev_tools/,
# *_tests.rs / test modules, and lines carrying an explicit `// ui-rules:` comment.
raw_widgets=$(rg -n \
  'ui\.checkbox|ui\.radio\b|ui\.toggle_value|ui\.button\(|ui\.selectable_label|ui\.text_edit_|ui\.menu_button|egui::ComboBox|egui::Slider|egui::RadioButton|egui::TextEdit|egui::ScrollArea' \
  "$repo_root/crates/app/src" "$repo_root/crates/ui/src" --glob '*.rs' \
  | rg -v '/components/|/dev_tools/|/tests/|_tests\.rs:|app_tests\.rs:|component_gallery_rendering\.rs:' \
  | rg -v 'ui-rules:' \
  | rg -v '#\[cfg\(test\)\]' || true)
if [[ -n "$raw_widgets" ]]; then
  echo "UI architecture check failed: raw egui widgets in view code (see RULES.md; use components/* or add a // ui-rules: exemption):" >&2
  echo "$raw_widgets" >&2
  exit 1
fi

# Widget layer boundary: components/, theme/ (incl. theme/tokens/), editor/ and
# diagram/ are pure widgets — they may depend on each other and on
# crate::{components,tokens,theme,i18n,policy,runtime,editor,diagram,result_grid,
# chart} plus crate-root items such as crate::DbProTheme, but must never import
# feature modules. `tokens/` is not a top-level dir (it lives inside theme/), so
# scanning theme/ covers it. Denylist covers three use-path shapes rustfmt can
# emit under imports_granularity="Crate". `dev_tools` and `query` are
# intentionally not listed (dev tooling / shared query widgets are permitted).
widget_layer_leaks=$(rg --pcre2 -U -n \
  'use crate::(app|agent|query_workbench|table|schema|explorer|files|shell|tools|settings|security|monitoring|palette|workspace|visual_query_builder|diagram_surface|component_gallery|connection)\b|use crate::\{\s*(app|agent|query_workbench|table|schema|explorer|files|shell|tools|settings|security|monitoring|palette|workspace|visual_query_builder|diagram_surface|component_gallery|connection)::|use crate::\{[^;{]*?[{,]\s*(app|agent|query_workbench|table|schema|explorer|files|shell|tools|settings|security|monitoring|palette|workspace|visual_query_builder|diagram_surface|component_gallery|connection)::' \
  "$repo_root/crates/ui/src/components" \
  "$repo_root/crates/ui/src/theme" \
  "$repo_root/crates/ui/src/editor" \
  "$repo_root/crates/ui/src/diagram" \
  --glob '*.rs' \
  | rg -v 'ui-rules:' \
  | rg -v '#\[cfg\(test\)\]' || true)
if [[ -n "$widget_layer_leaks" ]]; then
  echo "UI architecture check failed: widget layer (components/theme/editor/diagram/tokens) must not depend on feature modules." >&2
  echo "$widget_layer_leaks" >&2
  exit 1
fi

echo "UI architecture boundary: PASS"
