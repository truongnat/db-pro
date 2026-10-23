# Viewport Select and Radio Allocation Fit — Checklist

## 1. Analysis and Regression Testing
- [x] Reproduce horizontal overflow on Settings -> Appearance at baseline SHA.
- [x] Reproduce oversized picker geometry and duplicate ScrollArea ID on Query Context Picker at baseline SHA.
- [x] Write red regression test for horizontal Radio / Checkbox width allocation in `crates/ui/src/components/selection/tests/layout_bounds_tests.rs`.
- [x] Write red regression test for Query Context Picker width and height in `crates/app/src/query_workbench/tests/query_context_picker_view_tests.rs`.

## 2. Implementation
- [x] Implement compact width allocation in `crates/ui/src/components/selection/ui.rs` for `Radio` and `Checkbox`.
- [x] Propagate `compact(horizontal)` in `crates/ui/src/components/radio_group/ui.rs`.
- [x] Fix raw `ui.checkbox` in `crates/app/src/settings/settings_appearance_view.rs`.
- [x] Bound popup width, allocate each list region to its row count (capped at 180 pt) and give both `ScrollArea`s distinct `id_salt`s in `crates/app/src/query_workbench/query_context_picker_view.rs`.
- [x] Light palette: chrome one step below the canvas (`#f6f6f6`) with a preserved canvas → panel → hover → selected ladder, tertiary text raised to `#6b6b6b`, and transient popups moved to `surface_elevated`.
- [x] Add deterministic `DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER` capture hook in `crates/native-app/src/capture.rs` and `crates/app/src/workspace/workspace_actions.rs`.

## 3. Verification & Quality Gates
- [x] Unit tests pass: `cargo test -p db-pro-ui layout_bounds_tests` (4 passed).
- [x] Unit tests pass: `cargo test -p db-pro-app query_context_picker` (5 passed).
- [x] Run the 17-cell matrix (2 surfaces × 3 viewports × 2 themes + dense list + short list + Explorer popover) and verify each screenshot.
- [x] Verify `check-ui-architecture.sh` (`UI architecture boundary: PASS`).
- [x] Verify `cargo clippy --workspace --all-targets -- -D warnings` (exit 0, 0 diagnostics).
- [x] Verify `cargo check --workspace` (exit 0).
- [x] Verify `cargo +nightly fmt --all -- --check` (exit 0).
- [x] Full crate suites: `cargo test -p db-pro-ui -p db-pro-app` (725 + 681 passed, 0 failed).
- [x] Clean-code diff ratchet: 11 pass / 5 warn / 0 fail (exit 0).
- [x] **Mandatory native release gate** `cargo build --release --locked -p db-pro-native` — exit 0, re-run on the final tree.
- [x] Workspace gates: `cargo test --workspace` **2,272 passed / 0 failed / 65 ignored**.
- [x] The 65 `#[ignore]`d provider tests executed live: pg_integration 40/40, pg_object_identity 2/2, mysql_integration 12/12, sqlserver_integration 9/9, ssh_backup_runtime_verification 1/1, data_diff_smoke 1/1.
- [x] Narrow-viewport regression tests for compact row floor and popup viewport cap (red before, green after; both found real defects F5/F6).
- [x] Surface-ladder regression test so hover/selected can never collapse into the chrome fill.
- [x] Rebuild the capture binary and re-run the matrix so the evidence matches the final tree; every PNG re-inspected.
- [ ] CI "Rust checks" and an independent review — no PR was opened.
