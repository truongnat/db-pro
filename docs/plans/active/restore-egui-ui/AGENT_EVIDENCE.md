# Agent evidence — restore egui UI

## 1. Claim

Agent: Codex / implementation. Issues: n/a. Task state: Review. Baseline SHA: `8e1d33d1b21883a9b3dad12612d3cf74d3cbcd18`. Branch: main (owner override); PR: n/a, owner-authorized direct commit/push. Scope: temporarily remove rs-ui integration and restore established egui painting/behavior consistently with Query.

## 2. Progress checkpoint

Current source SHA: `02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f`. Executable integration, adapters, optional spike binary and dependencies removed. Automated gates pass. Larger-height framebuffer evidence is limited by the display; integration is deferred.

## 3. Implementation handoff / review request

Exact source SHA: `02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f` — revert(ui): restore egui and remove rs-ui integration. All source/manifests exactly match `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`. Inventory (restore/delete integration-only changes):

- `Cargo.lock`
- `crates/native-app/Cargo.toml`
- `crates/native-app/src/bin/rs-ui-welcome-spike.rs`
- `crates/native-app/src/capture.rs`
- `crates/native-app/src/main.rs`
- `crates/native-app/src/rs_ui_welcome_scene.rs`
- `crates/ui/Cargo.toml`
- `crates/ui/benches/result_grid_benchmarks.rs`
- `crates/ui/src/app_modules.rs`
- `crates/ui/src/app_tests.rs`
- `crates/ui/src/explorer_connection_node_view.rs`
- `crates/ui/src/explorer_connection_row_view.rs`
- `crates/ui/src/explorer_database_node_view.rs`
- `crates/ui/src/explorer_schema_feedback_view.rs`
- `crates/ui/src/explorer_schema_node_view.rs`
- `crates/ui/src/explorer_schema_objects_view.rs`
- `crates/ui/src/explorer_schema_tree_view.rs`
- `crates/ui/src/explorer_surface_view.rs`
- `crates/ui/src/explorer_table_folder_view.rs`
- `crates/ui/src/explorer_table_row_view.rs`
- `crates/ui/src/explorer_toolbar_view.rs`
- `crates/ui/src/explorer_tree.rs`
- `crates/ui/src/explorer_view.rs`
- `crates/ui/src/lib.rs`
- `crates/ui/src/native_explorer_paint.rs`
- `crates/ui/src/native_explorer_tree_runtime.rs`
- `crates/ui/src/native_runtime_shell.rs`
- `crates/ui/src/native_runtime_shell_tests.rs`
- `crates/ui/src/query_output_dock_surface_view.rs`
- `crates/ui/src/result_grid_body_view.rs`
- `crates/ui/src/result_grid_cell.rs`
- `crates/ui/src/result_grid_header.rs`
- `crates/ui/src/result_grid_header_surface_view.rs`
- `crates/ui/src/result_grid_selection.rs`
- `crates/ui/src/result_grid_view.rs`
- `crates/ui/src/result_grid_view_tests.rs`
- `crates/ui/src/result_grid_virtual_adapter.rs`
- `crates/ui/src/sidebar_surface_view.rs`
- `crates/ui/src/sidebar_view.rs`
- `crates/ui/src/table_data_state.rs`
- `crates/ui/src/theme.rs`
- `crates/ui/src/workspace_actions.rs`
- `crates/ui/src/workspace_feature_state.rs`
- `crates/ui/src/workspace_tab_primitives.rs`
- `crates/ui/src/workspace_tabs_surface_view.rs`

Acceptance and commands/counts: see VERIFICATION.md at this plan. CI: not run. Persisted-data/API migrations: none. Out of scope: independent rs-ui repository, SQL/domain/provider changes. Known limitations: requested 900/1080 logical heights clamped to 838; capture process auto-close issue at 1440; live drag/keyboard/database traversal not collected.

## 4. Review outcome

Reviewed SHA: `02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f`. Self-review verdict: ACCEPT WITH P2; introduced source P0=0/P1=0; open P2 evidence limitation: larger-height runtime captures. Inherited clean-code heuristic warnings: 2 categories; inherited native font replacement warnings are observable in capture logs. Independent review: not run; self-review is not independent approval. Inherited performance-scan benchmark invocation fails (`--quick` on libtest); the direct Criterion target passes. Native release binary is 32.0 MB. No performance improvement claim.

## 5. Research / audit handoff

n/a — implementation rollback; source and verification evidence are above.

## 6. Tổng kết (Vietnamese summary)

Đã đưa toàn bộ UI DB Pro về egui, giữ font/editor Query hiện tại và gỡ dependency/runtime/renderer rs-ui. Các gate Rust và build native pass; đã capture 15 ảnh. Màn hình giới hạn chiều cao capture lớn còn 838 logical px nên plan giữ RUNTIME_VERIFY. rs-ui integration được owner hoãn.
