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
- [x] Bound popup width and apply `auto_shrink([true, true])` + distinct `id_salt` in `crates/app/src/query_workbench/query_context_picker_view.rs`.
- [x] Add deterministic `DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER` capture hook in `crates/native-app/src/capture.rs` and `crates/app/src/workspace/workspace_actions.rs`.

## 3. Verification & Quality Gates
- [x] Unit tests pass: `cargo test -p db-pro-ui layout_bounds_tests` (4 passed).
- [x] Unit tests pass: `cargo test -p db-pro-app query_context_picker` (3 passed).
- [x] Run 12-cell matrix captures (2 surfaces × 3 viewports × 2 themes) and verify each screenshot.
- [x] Verify `check-ui-architecture.sh` (`UI architecture boundary: PASS`).
- [x] Verify `cargo clippy -p db-pro-ui -p db-pro-app -p db-pro-native --all-targets -- -D warnings` (exit 0, 0 errors).
- [x] Verify `cargo check -p db-pro-ui -p db-pro-app -p db-pro-native` (exit 0, fresh run after the narrow-viewport changes).
- [x] Verify `cargo +nightly fmt --all -- --check` (exit 0).
- [x] Full crate suites: `cargo test -p db-pro-ui -p db-pro-app` (679 + 724 passed, 0 failed).
- [x] Clean-code diff ratchet: 11 pass / 5 warn / 0 fail (exit 0).
- [x] **Mandatory native release gate** `cargo build --release --locked -p db-pro-native` — exit 0.
- [x] Workspace gates: `cargo test --workspace` **2,269 passed / 0 failed / 65 ignored**; `cargo clippy --workspace --all-targets -- -D warnings` 0 diagnostics; `cargo check --workspace` exit 0.
- [x] Re-verification on pushed HEAD `f17a1d4b`: all gates re-run, appearance capture reproduced byte-identically.
- [x] Narrow-viewport regression tests for compact row floor and popup viewport cap (red before, green after; both found real defects F5/F6).
- [x] Re-run clippy/fmt/check/tests after the narrow-viewport code changes.
- [x] Rebuild the capture binary and re-run the 12-cell matrix so the evidence matches the final tree; every PNG re-inspected.
