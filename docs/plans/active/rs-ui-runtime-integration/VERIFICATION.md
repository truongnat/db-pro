# Verification — rs-ui Runtime Integration

## Source evidence

- Stage 2 source commit: `948654524a6cd8a350a3eb972df33f2a5a1cd9ed`.
- At that DB Pro SHA, `native_runtime_shell.rs` keeps retained `Tab` nodes keyed to visible DB Pro tabs, mirrors selected/focused state and bounds, routes tab activation through rs-ui `Pressable`, and removes nodes for closed tabs. The same adapter routes sidebar and query-output dock drag deltas through rs-ui `Resizable`; `query_output_dock_surface_view.rs` preserves DB Pro's existing size setters and limits.
- DB Pro commit baseline: `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`.
- Earlier DB Pro implementation snapshot tree: `fa0b67697906a47570aca6af73dd339032182a7b` (isolated Git index containing `Cargo.lock` and `crates/ui`).
- rs-ui commit baseline: `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`.
- Exact local rs-ui working-tree source tree observed during this verification: `b66c61ac255ac4bcd1eb2c3d10e11abc2011886d`; the path dependency remains mutable and includes uncommitted sibling changes.
- `crates/native-app/src/main.rs` remains the eframe app entry point. DB Pro UI state and typed task bridge remain DB Pro-owned.
- `crates/ui/src/native_runtime_shell.rs` computes shell geometry using `UiTree` and owns sidebar `Resizable` and `ScrollState`. The Explorer scroll adapter preserves `codex_navigator_scroll`; the app-level test confirms rs-ui and egui offsets match after a wheel event.
- No database-facing behavior changed; PostgreSQL and SQLite impact is N/A.

## Automated checks

| Command | Result |
|---|---|
| `cargo test -p db-pro-ui native_runtime_shell::tests -- --nocapture` | PASS — 8 passed / 0 failed / 0 ignored, exit 0; includes tab semantics/activation, stale-node cleanup, and both output splitter directions. |
| `cargo test -p db-pro-ui query_output_dock_surface_view::tests -- --nocapture` | PASS — 1 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui --quiet` | PASS — 949 passed / 0 failed / 0 ignored, exit 0 |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::tests --no-fail-fast` | PASS — 5 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui sidebar_resize_routes_pointer_delta_through_rs_ui_runtime -- --nocapture` | PASS — 1 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui navigator_tree_scrolls_with_the_mouse_wheel -- --nocapture` | PASS — 1 passed / 0 failed / 0 ignored; verifies egui/rs-ui offsets match |
| `cargo check --workspace` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS against the final adapter source, exit 0 |
| `cargo test --workspace --quiet` | PASS — 1,603 passed / 0 failed / 41 ignored, exit 0 |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 14 pass / 2 warnings / 0 fail. Both warnings are existing long/parameter-heavy functions in `workspace_tabs_surface_view.rs`; no new adapter warning. |
| `git diff --check` | PASS, exit 0 |
| `cargo build --release --locked -p db-pro-native --features capture` | NOT RUN in this continuation; it passed during the prior shell-layout pass. |

## Runtime evidence

Existing screenshots from the shell-layout pass are retained in `screenshots/`:

| State | Requested logical viewport | Artifact | Observed result |
|---|---:|---|---|
| Empty | 1280×800 | `screenshots/shell-1280x800.png` | Captured at 2560×1600 physical pixels |
| Loading | 1280×800 | `screenshots/loading-1280x800.png` | Captured at 2560×1600 physical pixels |
| Connection error | 1280×800 | `screenshots/error-1280x800.png` | Captured at 2560×1600 physical pixels |
| Empty | 1440×900 | `screenshots/shell-1440x900.png` | Captured at 2880×1676; macOS limited logical height to 838 points |
| Any | 1920×1080 | none | NOT CAPTURED; prior capture attempt did not complete |

These images predate the sidebar resize/scroll and tab/splitter behavior adapters. They do not verify those interactions. Runtime unit tests exercise adapter state and normalized activation, but do not prove OS-level pointer capture, live DB connections, or keyboard focus in the running native app.

Interactive smoke for opening a connection, browsing schema, executing a query, switching tabs, scrolling results, pane resize, and keyboard focus remains pending.

## Clean-code scan notes

The two ratcheted warnings are in the existing workspace tab rendering path touched only to consume the rs-ui tab height: `draw_workspace_tabs` and tab helper functions in `crates/ui/src/workspace_tabs_surface_view.rs`. This integration does not refactor that behavior.

## Remaining limitations

- Sidebar and query-output splitter resize use rs-ui `Resizable`; egui remains responsible for pointer capture and painting. Tab selection activation goes through rs-ui `Pressable`, while DB Pro retains the existing action dispatch.
- Close-button activation and roving arrow-key tab navigation are not routed through rs-ui. Native keyboard focus/accessibility behavior remains unverified.
- Schema tree, result grid, and search/filter migrations are not implemented.
- Local sibling rs-ui sources are dirty and the dependency is not pinned for clean CI/release reproducibility.
- Full rs-ui renderer/window integration is not implemented; DB Pro remains on eframe/egui glow.
- 1920×1080 capture and native end-to-end smoke remain pending.
