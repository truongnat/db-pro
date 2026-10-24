---
schema_version: 1
id: "MEM-PLAN-restore-egui-ui-39cd9d723d6a"
title: "Plan archive: Restore Egui Ui"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, restore-egui-ui]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "39cd9d723d6a6fe02b8e134ee1542f63a3d753ecc0a826122cf91824c60512a5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/restore-egui-ui/PLAN.md"
  - "docs/plans/active/restore-egui-ui/FINDINGS.md"
  - "docs/plans/active/restore-egui-ui/VERIFICATION.md"
  - "docs/plans/active/restore-egui-ui/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/restore-egui-ui/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Restore egui UI

State: RUNTIME_VERIFY
Branch: main (owner override)

Owner request (2026-10-02): temporarily remove all rs-ui integration and use egui consistently with the Query editor. Restore the established egui source/manifests from `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`; preserve domain, query, database, persistence and editor implementations. Remove the rs-ui adapters, Explorer texture renderer, experimental binary and dependencies. Keep the independent rs-ui repository intact.

Acceptance: no rs-ui references in executable Rust/manifests/lockfile; sidebar, tabs, resizing and grid return to egui; workspace gates and release build pass; capture normal/loading/error/empty at 1280×800, 1440×900 and 1920×1080.


## Source: `FINDINGS.md`

# Findings

Owner chose the existing egui Query appearance as the UI baseline. Integration commits after `c0c1f5525b20a810913d1eee13c7ee2dd15b6664` exclusively introduced rs-ui adapters/rendering and their support code; restoring the affected source/manifests to that revision is a coherent rollback. No history reset is needed. Existing rs-ui implementation evidence remains historical; migration is deferred.

Provider impact: PostgreSQL/SQLite backend code unchanged; no new provider behavior claimed. Preserve egui virtual row windows, persisted column widths and existing selection ownership. No performance improvement claim is made without measurements.

Reusable lesson: when a contiguous integration is deferred, restore its affected source/manifests from the verified pre-integration SHA and compare the entire code path to that SHA; preserve historical evidence and mark it deferred instead of resetting history. This task demonstrates an empty source diff against the reference SHA.


## Source: `VERIFICATION.md`

# Verification

State: RUNTIME_VERIFY
Source SHA: `02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f`
Restored implementation SHA: `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`
`git diff c0c1f5525b20a810913d1eee13c7ee2dd15b6664 02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f -- crates Cargo.lock Cargo.toml` is empty: all shipping Rust/manifests match the pre-integration egui implementation. No `rs_ui` or `rs-ui` remains in Rust/manifests/lockfile.

## Automated gates

- `git diff --check`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `cargo check --workspace`: exit 0.
- `cargo test -p db-pro-ui`: 940 passed / 0 failed / 0 ignored.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace`: exit 0; 1597 passed / 0 failed / 41 ignored across 22 suite summaries. Ignored provider tests are not runtime proof.
- `cargo build --release --locked -p db-pro-native --features capture`: exit 0.
- `cargo build --release --locked -p db-pro-native`: exit 0 (executed by native perf scan); binary 32.0 MB.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: exit 0; 14 passes, 2 inherited heuristic warnings (3 functions with >3 arguments, 11 long functions). Restored source matches the established baseline; no new helper/module was introduced.
- `bash .skills/perf-audit/scripts/perf-scan.sh native`: exit 1; native build and 32.0 MB size checks passed, benchmark dispatch failed because `--quick` was forwarded to the libtest harness (`Unrecognized option: quick`). Inherited tooling defect; script unchanged.
- `cargo bench -p db-pro-ui --bench result_grid_benchmarks -- --quick`: exit 0; direct Criterion target completed. See evidence/grid-bench.log for measured intervals. No comparative improvement claim.

## Native runtime evidence

Capture-feature release binary using isolated temporary `DB_PRO_DATA_DIR`, `DB_PRO_CAPTURE_TO`, `DB_PRO_WINDOW_SIZE`, settle 30 frames. Normal uses Query/light helper; table uses deterministic typed-cell fixture; empty has no connections; loading shows pending connection request; error opens deterministic validation-error dialog. These are native egui framebuffers, not CPU mock previews. PostgreSQL/SQLite live database flows were not exercised; their code was unchanged.

Retina scale: 2.0. Requested logical widths were honored; requested heights 900/1080 were clamped by macOS to 838. Formal larger-height acceptance remains open, so this plan stays active. Native capture exits were 0 for 1280 and 1920 scenes; 1440 scenes wrote their PNG but did not auto-close, so the isolated capture processes were terminated after capture (first normal scene timed out at 45 s). No user app process was terminated. No live keyboard/drag/provider claim is made from these fixtures.

| Screenshot | Framebuffer pixels | Actual logical viewport |
|---|---|---|
| empty-1280x800.png | 2560×1600 | 1280×800 |
| empty-1440x900.png | 2880×1676 | 1440×838 |
| empty-1920x1080.png | 3840×1676 | 1920×838 |
| error-1280x800.png | 2560×1600 | 1280×800 |
| error-1440x900.png | 2880×1676 | 1440×838 |
| error-1920x1080.png | 3840×1676 | 1920×838 |
| loading-1280x800.png | 2560×1600 | 1280×800 |
| loading-1440x900.png | 2880×1676 | 1440×838 |
| loading-1920x1080.png | 3840×1676 | 1920×838 |
| normal-1280x800.png | 2560×1600 | 1280×800 |
| normal-1440x900.png | 2880×1676 | 1440×838 |
| normal-1920x1080.png | 3840×1676 | 1920×838 |
| table-1280x800.png | 2560×1600 | 1280×800 |
| table-1440x900.png | 2880×1676 | 1440×838 |
| table-1920x1080.png | 3840×1676 | 1920×838 |


## Source: `AGENT_EVIDENCE.md`

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

