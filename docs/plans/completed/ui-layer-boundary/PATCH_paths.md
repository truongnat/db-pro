# Patch: path references (PLAN §1 + §3 path-table portion)

All new paths verified against the post-reorg tree (`find crates/ui/src -name '*.rs'`).
Blocks are disjoint from `PatchGlobs` (loops at L310, L346-355, L366-387;
they confirmed they use `rg --files -g`/`--glob`, no `**` shell globbing)
and `PatchLayer` (widget-layer rule).

# Changes

## 1. Bash-glob safety note (no code change required by this patch)

My patch avoids `**` shell globs entirely — `state_field_leaks` uses
`rg --glob '*_state.rs'` instead, which recurses portably (bash 3.2-safe,
no `shopt -s globstar` needed). If ANY sibling patch introduces a `**`
bash glob, the integrator must add `shopt -s globstar` after
`set -euo pipefail` (L2); skip it otherwise.

## 2. Composition-root + events file variables

**L5-6:**

Old:
```bash
app_file="$repo_root/crates/ui/src/app.rs"
events_file="$repo_root/crates/ui/src/events.rs"
```

New:
```bash
app_file="$repo_root/crates/ui/src/app/mod.rs"
events_file="$repo_root/crates/ui/src/runtime/events.rs"
```

## 3. Error messages naming `app.rs` (file is now `app/mod.rs`)

**L9:**
Old: `  echo "UI architecture check failed: app.rs must include the isolated module topology registry." >&2`
New: `  echo "UI architecture check failed: app/mod.rs must include the isolated module topology registry." >&2`

**L13:**
Old: `  echo "UI architecture check failed: app.rs must not own feature module path declarations." >&2`
New: `  echo "UI architecture check failed: app/mod.rs must not own feature module path declarations." >&2`

**L17:**
Old: `  echo "UI architecture check failed: app.rs must delegate runtime command construction to feature adapters." >&2`
New: `  echo "UI architecture check failed: app/mod.rs must delegate runtime command construction to feature adapters." >&2`

**L22:**
Old: `  echo "UI architecture check failed: app.rs must remain a composition root without egui painting." >&2`
New: `  echo "UI architecture check failed: app/mod.rs must remain a composition root without egui painting." >&2`

## 4. Event router variable

**L68:**

Old:
```bash
router_file="$repo_root/crates/ui/src/event_router.rs"
```

New:
```bash
router_file="$repo_root/crates/ui/src/runtime/event_router.rs"
```

## 5. drain_events bound check

**L74:**

Old:
```bash
if ! rg -q 'drain_events\(crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME\)' "$repo_root/crates/ui/src/events.rs"; then
```

New:
```bash
if ! rg -q 'drain_events\(crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME\)' "$repo_root/crates/ui/src/runtime/events.rs"; then
```

Verified: `runtime/events.rs:25` contains the call.

## 6. task_bridge direct-call exemptions — composition root is now `app/mod.rs`

Three exclusions filter `/app\.rs:`; the only files calling
`task_bridge.send_best_effort(` / `task_bridge.next_request_id()` today is
`app/mod.rs` (verified via rg — no `.send(` callers exist at all, so the same
narrow exclusion keeps parity). Do NOT widen to `/app/` — `app/tests/` and
`app/app_state.rs` must stay covered.

**L79:**
Old: `| rg -v '/app\.rs:' || true)` → New: `| rg -v '/app/mod\.rs:' || true)`

**L86:**
Old: `| rg -v '/app\.rs:' || true)` → New: `| rg -v '/app/mod\.rs:' || true)`

**L93:**
Old: `| rg -v '/app\.rs:' || true)` → New: `| rg -v '/app/mod\.rs:' || true)`

(Context lines for all three, verbatim old → new:)

```bash
direct_sends=$(rg --pcre2 -U -n 'task_bridge\s*(?:\n\s*)?\.send\(' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app\.rs:' || true)
```
→
```bash
direct_sends=$(rg --pcre2 -U -n 'task_bridge\s*(?:\n\s*)?\.send\(' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app/mod\.rs:' || true)
```

```bash
direct_best_effort_sends=$(rg --pcre2 -U -n 'task_bridge\s*(?:\n\s*)?\.send_best_effort\(' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app\.rs:' || true)
```
→
```bash
direct_best_effort_sends=$(rg --pcre2 -U -n 'task_bridge\s*(?:\n\s*)?\.send_best_effort\(' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app/mod\.rs:' || true)
```

```bash
direct_request_allocations=$(rg -n 'task_bridge\.next_request_id\(\)' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app\.rs:' || true)
```
→
```bash
direct_request_allocations=$(rg -n 'task_bridge\.next_request_id\(\)' "$repo_root/crates/ui/src" --glob '*.rs' | rg -v '/app/mod\.rs:' || true)
```

## 7. saved_tasks_file

**L107:**

Old:
```bash
saved_tasks_file="$repo_root/crates/ui/src/tasks_view.rs"
```

New:
```bash
saved_tasks_file="$repo_root/crates/ui/src/tools/tasks_view.rs"
```

Verified: `tools/tasks_view.rs:276` uses `saved_task_sql::build_export_query`, so
both the ban and the positive check still hold.

## 8. database_operations_state deletion guard — keep old AND new path

**L117-120:**

Old:
```bash
if [[ -e "$repo_root/crates/ui/src/database_operations_state.rs" ]]; then
  echo "UI architecture check failed: database_operations_state.rs catch-all must stay deleted." >&2
  exit 1
fi
```

New:
```bash
if [[ -e "$repo_root/crates/ui/src/database_operations_state.rs" ]]; then
  echo "UI architecture check failed: database_operations_state.rs catch-all must stay deleted." >&2
  exit 1
fi
if [[ -e "$repo_root/crates/ui/src/schema/database_operations_state.rs" ]]; then
  echo "UI architecture check failed: schema/database_operations_state.rs catch-all must stay deleted." >&2
  exit 1
fi
```

Verified: neither path exists today.

## 9. TableState data-lifecycle ban

**L122:**

Old:
```bash
if rg -n 'table_data|row_reload' "$repo_root/crates/ui/src/table_state.rs"; then
```

New:
```bash
if rg -n 'table_data|row_reload' "$repo_root/crates/ui/src/table/table_state.rs"; then
```

## 10. activity_bar_view WorkspaceFeatureState ban

**L127:**

Old:
```bash
if rg -n '&mut WorkspaceFeatureState|WorkspaceFeatureState' "$repo_root/crates/ui/src/activity_bar_view.rs"; then
```

New:
```bash
if rg -n '&mut WorkspaceFeatureState|WorkspaceFeatureState' "$repo_root/crates/ui/src/shell/activity_bar_view.rs"; then
```

Verified clean today (no match → check passes).

## 11. TableDataQueryState existence check

**L131:**

Old:
```bash
if [[ ! -f "$repo_root/crates/ui/src/table_data_query_state.rs" ]]; then
```

New:
```bash
if [[ ! -f "$repo_root/crates/ui/src/table/table_data_query_state.rs" ]]; then
```

## 12. explicit_state_modules — full verbatim replacement

**L136-301.** Every entry from the old list reappears under its folder (164
files, same order, all verified to exist). Also fixes the pre-existing
indentation typo at L198 (`palette_search_view.rs` was indented one space).

Old (header + first/last entries shown; replace the whole array):
```bash
explicit_state_modules=(
  "$repo_root/crates/ui/src/activity_bar_view.rs"
  "$repo_root/crates/ui/src/workspace_session.rs"
  ... ~162 more flat-path entries ...
  "$repo_root/crates/ui/src/table_data_mutation_toolbar_view.rs"
)
```

New (verbatim):
```bash
explicit_state_modules=(
  "$repo_root/crates/ui/src/shell/activity_bar_view.rs"
  "$repo_root/crates/ui/src/workspace/workspace_session.rs"
  "$repo_root/crates/ui/src/connection/view.rs"
  "$repo_root/crates/ui/src/connection/form_fields.rs"
  "$repo_root/crates/ui/src/connection/advanced_panels.rs"
  "$repo_root/crates/ui/src/connection/connection_status.rs"
  "$repo_root/crates/ui/src/connection/connection_events.rs"
  "$repo_root/crates/ui/src/table/table_editor_context.rs"
  "$repo_root/crates/ui/src/table/table_data_state.rs"
  "$repo_root/crates/ui/src/table/table_data_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_data_query_state.rs"
  "$repo_root/crates/ui/src/table/table_editing_state.rs"
  "$repo_root/crates/ui/src/table/table_editor_values.rs"
  "$repo_root/crates/ui/src/visual_query_builder/visual_query_builder_state.rs"
  "$repo_root/crates/ui/src/visual_query_builder/visual_query_builder_surface_view.rs"
  "$repo_root/crates/ui/src/agent/agent_context.rs"
  "$repo_root/crates/ui/src/agent/agent_confirmation.rs"
  "$repo_root/crates/ui/src/agent/agent_context_actions_view.rs"
  "$repo_root/crates/ui/src/agent/agent_header_view.rs"
  "$repo_root/crates/ui/src/agent/agent_surface_view.rs"
  "$repo_root/crates/ui/src/agent/agent_thread_surface_view.rs"
  "$repo_root/crates/ui/src/tools/audit_surface_view.rs"
  "$repo_root/crates/ui/src/tools/event_trigger_surface_view.rs"
  "$repo_root/crates/ui/src/tools/fdw_surface_view.rs"
  "$repo_root/crates/ui/src/files/files_surface_view.rs"
  "$repo_root/crates/ui/src/files/files_secondary_tabs_view.rs"
  "$repo_root/crates/ui/src/tools/pg_settings_surface_view.rs"
  "$repo_root/crates/ui/src/tools/replication_surface_view.rs"
  "$repo_root/crates/ui/src/monitoring/monitoring_surface_view.rs"
  "$repo_root/crates/ui/src/agent/agent_patch.rs"
  "$repo_root/crates/ui/src/agent/agent_result_projection.rs"
  "$repo_root/crates/ui/src/explorer/schema_explorer_state.rs"
  "$repo_root/crates/ui/src/agent/agent_settings_view.rs"
  "$repo_root/crates/ui/src/settings/settings_appearance_view.rs"
  "$repo_root/crates/ui/src/settings/settings_backup_view.rs"
  "$repo_root/crates/ui/src/agent/agent_workflow_reducer.rs"
  "$repo_root/crates/ui/src/tools/synthetic_data.rs"
  "$repo_root/crates/ui/src/tools/masking.rs"
  "$repo_root/crates/ui/src/security/security_rls.rs"
  "$repo_root/crates/ui/src/monitoring/monitoring_state.rs"
  "$repo_root/crates/ui/src/monitoring/monitoring_snapshot_view.rs"
  "$repo_root/crates/ui/src/tools/audit_state.rs"
  "$repo_root/crates/ui/src/schema/routine_state.rs"
  "$repo_root/crates/ui/src/tools/transfer_state.rs"
  "$repo_root/crates/ui/src/tools/transfer_harness_view.rs"
  "$repo_root/crates/ui/src/tools/transfer_activity_surface_view.rs"
  "$repo_root/crates/ui/src/tools/saved_tasks_surface_view.rs"
  "$repo_root/crates/ui/src/tools/synthetic_data_state.rs"
  "$repo_root/crates/ui/src/tools/masking_state.rs"
  "$repo_root/crates/ui/src/tools/pg_settings_state.rs"
  "$repo_root/crates/ui/src/tools/fdw_state.rs"
  "$repo_root/crates/ui/src/tools/replication_state.rs"
  "$repo_root/crates/ui/src/tools/event_trigger_state.rs"
  "$repo_root/crates/ui/src/security/security_state.rs"
  "$repo_root/crates/ui/src/schema/database_management_state.rs"
  "$repo_root/crates/ui/src/schema/schema_workspace_state.rs"
  "$repo_root/crates/ui/src/diagram_surface/diagram_view.rs"
  "$repo_root/crates/ui/src/diagram_surface/diagram_canvas_view.rs"
  "$repo_root/crates/ui/src/diagram_surface/diagram_design_actions.rs"
  "$repo_root/crates/ui/src/diagram_surface/diagram_design_panel_view.rs"
  "$repo_root/crates/ui/src/palette/palette_catalog.rs"
  "$repo_root/crates/ui/src/palette/palette_search_view.rs"
  "$repo_root/crates/ui/src/palette/palette_surface_view.rs"
  "$repo_root/crates/ui/src/files/files_agent_context_view.rs"
  "$repo_root/crates/ui/src/files/files_tree_view.rs"
  "$repo_root/crates/ui/src/files/files_search_view.rs"
  "$repo_root/crates/ui/src/files/files_tasks_view.rs"
  "$repo_root/crates/ui/src/files/files_git_view.rs"
  "$repo_root/crates/ui/src/settings/settings_system_view.rs"
  "$repo_root/crates/ui/src/settings/settings_surface_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_data_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_chrome_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_surface_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_queries_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_problems_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_query_library_view.rs"
  "$repo_root/crates/ui/src/shell/sidebar_query_shortcuts_view.rs"
  "$repo_root/crates/ui/src/shell/shell_frame_view.rs"
  "$repo_root/crates/ui/src/shell/shell_topbar_view.rs"
  "$repo_root/crates/ui/src/shell/shell_statusbar_view.rs"
  "$repo_root/crates/ui/src/shell/welcome_surface_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_toolbar_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_connection_row_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_connection_node_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_database_node_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_schema_feedback_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_dialog_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_save_dialog_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_transaction_surface_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_schema_node_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_schema_object_row_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_schema_object_folders_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_schema_objects_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_schema_tree_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_surface_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_navigation.rs"
  "$repo_root/crates/ui/src/explorer/explorer_table_row_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_table_details_view.rs"
  "$repo_root/crates/ui/src/explorer/explorer_table_folder_view.rs"
  "$repo_root/crates/ui/src/table/table_workspace_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_snippets.rs"
  "$repo_root/crates/ui/src/query_workbench/query_diagnostics_view.rs"
  "$repo_root/crates/ui/src/shell/navigation_view.rs"
  "$repo_root/crates/ui/src/tools/maintenance_activity_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_export.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_selection.rs"
  "$repo_root/crates/ui/src/schema/schema_compare_view.rs"
  "$repo_root/crates/ui/src/schema/schema_object_surface_view.rs"
  "$repo_root/crates/ui/src/schema/schema_object_resolver.rs"
  "$repo_root/crates/ui/src/schema/schema_workbench_form.rs"
  "$repo_root/crates/ui/src/schema/schema_workbench_mutation.rs"
  "$repo_root/crates/ui/src/schema/schema_workbench_secondary_view.rs"
  "$repo_root/crates/ui/src/schema/schema_workbench_surface_view.rs"
  "$repo_root/crates/ui/src/security/security_confirmation_view.rs"
  "$repo_root/crates/ui/src/security/security_surface_view.rs"
  "$repo_root/crates/ui/src/settings/settings_navigation_view.rs"
  "$repo_root/crates/ui/src/settings/settings_keybindings_view.rs"
  "$repo_root/crates/ui/src/settings/settings_diagnostics_view.rs"
  "$repo_root/crates/ui/src/settings/settings_general_view.rs"
  "$repo_root/crates/ui/src/settings/settings_editor_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_search_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_shell_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_context_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_context_picker_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_parameters_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_completion_popup_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_editor_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_actions_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_snippets_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_run_control_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_explain_actions.rs"
  "$repo_root/crates/ui/src/query_workbench/query_save_actions.rs"
  "$repo_root/crates/ui/src/query_workbench/query_output_tabs_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_output_dock_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_output_panes_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_output_actions_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_layout_surface_view.rs"
  "$repo_root/crates/ui/src/query_workbench/query_results_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_toolbar_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_body_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_header_menu_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_header_content_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_header_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_interaction_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_keyboard_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_row_gutter_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_row_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_cell_menu_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_cell_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_cell_editor_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_inspector_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_record_surface_view.rs"
  "$repo_root/crates/ui/src/workspace/workspace_tabs_surface_view.rs"
  "$repo_root/crates/ui/src/result_grid/result_grid_projection.rs"
  "$repo_root/crates/ui/src/table/table_data_placeholder_view.rs"
  "$repo_root/crates/ui/src/table/table_structure_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_scroll_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_indexes_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_relations_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_metadata_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_profile_surface_view.rs"
  "$repo_root/crates/ui/src/table/table_data_pagination_view.rs"
  "$repo_root/crates/ui/src/table/table_data_filter_view.rs"
  "$repo_root/crates/ui/src/table/table_data_mutation_toolbar_view.rs"
)
```

Non-obvious folder assignments (verified by `find`, all paths exist):
- `diagram_view`/`diagram_canvas_view`/`diagram_design_*` → `diagram_surface/`
  (the `diagram/` folder is the pure renderer, not these view files)
- `audit_*`, `fdw_*`, `pg_settings_*`, `replication_*`, `event_trigger_*`,
  `transfer_*`, `saved_tasks_*`, `synthetic_data*`, `masking*`,
  `maintenance_activity_view` → `tools/`
- `security_rls`/`security_state`/`security_*_view` → `security/`
- `schema_explorer_state` → `explorer/`
- `connection_status`, `connection_events` → `connection/`
- All `query_*`/`sidebar_*`/`shell_*`/`navigation_view`/`activity_bar_view`/
  `welcome_surface_view` → `query_workbench/` resp. `shell/`
- `database_management_state`, `routine_state`, `schema_workspace_state`,
  `schema_compare_view`, `schema_object_*`, `schema_workbench_*` → `schema/`
- `workspace_session`, `workspace_tabs_surface_view` → `workspace/`
- `maintenance_activity_view` → `tools/`
- `table_workspace_surface_view` → `table/`

COORDINATION (resolved): PatchGlobs conceded — their proposed
`*_state.rs` + `*_surface_view.rs` glob covered only ~91 files and dropped
~73 gated modules (`agent_context`, `explorer_navigation`,
`query_save_actions`, `security_rls`, `synthetic_data`, `workspace_session`,
reducers/actions/projections…). This verbatim 164-entry list is
authoritative; their patch marks this block NOT MINE.

## 13. `*_state.rs` DbProApp loop — exemption path only

**L311** (L310's `src/*_state.rs` glob line belongs to PatchGlobs'
`rg --files -g` rewrite; this is the only path inside it):

Old:
```bash
  [[ "$state_module" == "$repo_root/crates/ui/src/app_state.rs" ]] && continue
```

New:
```bash
  [[ "$state_module" == "$repo_root/crates/ui/src/app/app_state.rs" ]] && continue
```

(PatchGlobs confirms their rewritten L310 loop already contains the
repointed `app/app_state.rs` exemption — whichever patch lands first gives
the same result; this entry kept for completeness.)

## 14. UiCommand bans — seven single-file checks

**L318:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/table_state.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/table/table_state.rs"; then`

**L322:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/table_data_query_state.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/table/table_data_query_state.rs"; then`

**L326:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_library_state.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_workbench/query_library_state.rs"; then`

**L330:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/schema_workbench.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/schema/schema_workbench.rs"; then`

**L334:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/explorer_navigation.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/explorer/explorer_navigation.rs"; then`

**L338:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_execution_actions.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_workbench/query_execution_actions.rs"; then`

**L342:**
Old: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_save_actions.rs"; then`
New: `if rg -n '\bUiCommand\b' "$repo_root/crates/ui/src/query_workbench/query_save_actions.rs"; then`

NOTE (resolved): the three `*_state.rs` checks (L318/L322/L326) are
strictly subsumed by PatchGlobs' recursive `*_state.rs` UiCommand glob
(verified: zero `*_state.rs` files contain UiCommand). If that glob lands,
integrator deletes these three checks; the four non-state repoints above
are still required.

## 15. state_field_leaks glob + explicit files

**L393-398:**

Old:
```bash
state_field_leaks=$(rg -n '^\s*pub(\(crate\))? [A-Za-z_][A-Za-z0-9_]*:' \
  "$repo_root/crates/ui/src"/*_state.rs \
  "$repo_root/crates/ui/src/schema_workbench.rs" \
  "$repo_root/crates/ui/src/connection/state.rs" \
  "$repo_root/crates/ui/src/connection/lifecycle.rs" \
  "$repo_root/crates/ui/src/connection/catalog.rs" || true)
```

New:
```bash
state_field_leaks=$(rg -n '^\s*pub(\(crate\))? [A-Za-z_][A-Za-z0-9_]*:' \
  "$repo_root/crates/ui/src" --glob '*_state.rs' \
  "$repo_root/crates/ui/src/schema/schema_workbench.rs" \
  "$repo_root/crates/ui/src/connection/state.rs" \
  "$repo_root/crates/ui/src/connection/lifecycle.rs" \
  "$repo_root/crates/ui/src/connection/catalog.rs" || true)
```

`--glob '*_state.rs'` scopes the recursive `src` scan to state files (40
today, incl. `app/app_state.rs`, which has no `pub`/`pub(crate)` plain
fields — verified) while explicit file args bypass the glob and are always
searched (verified: `connection/catalog.rs` hits `pub(in crate::app)` with
the glob in place). No globstar needed. Keeps `connection/catalog.rs` from
the original (unchanged path, still exists); PLAN's "three explicit files"
summary omitted it but dropping it would silently lose coverage.

NOTE (resolved): PatchGlobs conceded this block too — this `rg --glob`
version is authoritative (simpler than a shared array; no globstar needed).

## 16. Event-module existence loop

**L413-418.** Old list had 17 flat module names. New: 3 runtime modules +
14 feature `*_events.rs` files (the same 13 feature modules as before,
at their new folders, PLUS `query_workbench/query_failure_events.rs` which
exists today and was missing from the old existence check).

Old:
```bash
for module in event_router agent_events connection_events ddl_events file_picker_events management_events operation_events schema_events table_events query_library_events query_prediction_events query_save_events query_history_events query_queue_events query_execution_events query_result_events query_multi_result_events; do
  test -f "$repo_root/crates/ui/src/${module}.rs" || {
    echo "UI architecture check failed: missing event module ${module}.rs." >&2
    exit 1
  }
done
```

New:
```bash
for event_module in \
  "$repo_root/crates/ui/src/runtime/events.rs" \
  "$repo_root/crates/ui/src/runtime/event_router.rs" \
  "$repo_root/crates/ui/src/runtime/operation_events.rs" \
  "$repo_root/crates/ui/src/agent/agent_events.rs" \
  "$repo_root/crates/ui/src/connection/connection_events.rs" \
  "$repo_root/crates/ui/src/files/file_picker_events.rs" \
  "$repo_root/crates/ui/src/tools/management_events.rs" \
  "$repo_root/crates/ui/src/schema/schema_events.rs" \
  "$repo_root/crates/ui/src/table/ddl_events.rs" \
  "$repo_root/crates/ui/src/table/table_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_library_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_prediction_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_save_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_history_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_queue_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_execution_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_result_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_multi_result_events.rs" \
  "$repo_root/crates/ui/src/query_workbench/query_failure_events.rs"; do
  test -f "$event_module" || {
    echo "UI architecture check failed: missing event module ${event_module#$repo_root/crates/ui/src/}." >&2
    exit 1
  }
done
```

NOTE (resolved): PatchGlobs conceded this block — this 19-path explicit
list is authoritative (stricter than a non-empty glob; verified all paths
exist).

## 17. Verified unchanged — no edit needed

- **L405-406 `pub(in crate::app)` check on `src/connection/state.rs`** — path
  unchanged, file exists; `crate::app` still resolves (dir module). Currently
  zero matches → check passes. `connection/catalog.rs` and `connection/lifecycle.rs`
  do contain `pub(in crate::app)` but only `state.rs` was ever gated; keep
  semantics.
- **L361-364 `UiCommand` ban on `src/connection/lifecycle.rs`** — path
  unchanged, file exists, currently clean.
- **L100 `egui::Window::new` src-wide scan** — path-independent, still valid.
- **L388 `database_operations` name scan** — src-wide, covers new paths.

# Out of my scope (owned by siblings, noted for integrator)

- L310 `for state_module in "$repo_root"/crates/ui/src/*_state.rs` glob →
  `rg --files -g` rewrite (PatchGlobs); my change 13 fixes the exemption
  line inside it.
- L346-355 nine-file `*_state.rs` UiCommand loop → recursive `*_state.rs`
  glob (PatchGlobs); may also subsume my L318/L322/L326 repoints (see §14).
- L366-387 reducer list → `src/**/*_events.rs` minus `src/runtime/*` (PatchGlobs).
- L423-428 raw-widgets exclusion `app_tests.rs:` → add `*/tests/` dirs (PatchGlobs).
- Widget-layer boundary rule (PatchLayer).

# Verification

- Ran `find crates/ui/src -name '*.rs'` and checked every new path in this
  patch: all 164 `explicit_state_modules` entries + all moved check targets
  exist.
- Ran targeted `rg` on moved files: `tools/tasks_view.rs` contains
  `build_export_query` (L276); `table/table_state.rs` has no
  `table_data|row_reload|UiCommand`; `shell/activity_bar_view.rs` has no
  `WorkspaceFeatureState`; `runtime/events.rs:25` has the bounded
  `drain_events` call; `app/mod.rs` has `include!("app_modules.rs")` and no
  `#[path`/`UiCommand::`.
- `pub(in crate::app)` appears in `connection/catalog.rs` +
  `connection/lifecycle.rs` but not `connection/state.rs` → the check still
  applies and passes.
- Verified `rg -l 'UiCommand' -g '*_state.rs'` → zero matches, so a
  recursive `*_state.rs` UiCommand glob (PatchGlobs) is a safe superset
  of the three individual state-file bans.
- Verified `rg --glob '*_state.rs'` + explicit file args: explicit files
  bypass the glob (catalog.rs `pub(in crate::app)` still found); the glob
  scopes the recursive scan to the 40 `*_state.rs` files.
- Parent should run: `bash scripts/check-ui-architecture.sh` after all three
  patches land (expect PASS), plus the negative tests in PLAN Acceptance.
