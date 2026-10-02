# Verification — rs-ui Runtime Integration

## Source evidence

- Stage 2 source commits: `948654524a6cd8a350a3eb972df33f2a5a1cd9ed` and `42f14e520fa1e8bb280c1ec0399d3f523d4792da`.
- At DB Pro SHA `42f14e520fa1e8bb280c1ec0399d3f523d4792da`, `native_runtime_shell.rs` keeps retained `Tab` nodes keyed to visible DB Pro tabs, mirrors selected/focused state and bounds, routes tab and close-button activation through rs-ui `Pressable`, and removes nodes for closed tabs. Close buttons are semantic `Button` children of their tab. The same adapter routes sidebar and query-output dock drag deltas through rs-ui `Resizable`; `query_output_dock_surface_view.rs` preserves DB Pro's existing size setters and limits.
- DB Pro commit baseline: `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`.
- Earlier DB Pro implementation snapshot tree: `fa0b67697906a47570aca6af73dd339032182a7b` (isolated Git index containing `Cargo.lock` and `crates/ui`).
- rs-ui commit baseline: `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`.
- Available rs-ui checkout for this continuation: clean `main` at `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`.
- `crates/native-app/src/main.rs` remains the eframe app entry point. DB Pro UI state and typed task bridge remain DB Pro-owned.
- `crates/ui/src/native_runtime_shell.rs` computes shell geometry using `UiTree` and owns sidebar `Resizable` and `ScrollState`. The Explorer scroll adapter preserves `codex_navigator_scroll`; the app-level test confirms rs-ui and egui offsets match after a wheel event.
- No database-facing behavior changed; PostgreSQL and SQLite impact is N/A.
- DB Pro baseline before the Stage 4 continuation: `5c1f42bad87c69f3cbfddcb921c557987706201e`; Stage 4 implementation source: `b224ec376ee50a93516d2c5691b700bcd9c63f8f`. At rs-ui SHA `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`, `VirtualGrid` and `ScrollState` are exported; `VirtualGrid::new_fixed` accepts one fixed column extent, while `ui-runtime` has no `SelectionModel` or `Resizable`. The same rs-ui revision lacks `BehaviorCommand`, `ResizeAxis`, `ResizeConfig`, `dispatch_behavior_command`, and `register_resizable`, which are already referenced by DB Pro Stages 1–3. A pin to that SHA therefore fails compilation and was not retained.
- Stage 4 working-tree adapter calculates a vertical visible row range with rs-ui and leaves egui as input host/painter; DB Pro still owns projection, typed cells, widths and selection. Horizontal virtualization and rs-ui selection/resize routing remain incomplete because the available API cannot preserve variable-width columns or existing interactions.

### Stage 3 — Explorer tree adapter

- Initial source SHA: `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7`; current Stage 3 implementation/refactor SHA: `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8`.
- At current SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8`, `crates/ui/src/native_explorer_tree_runtime.rs:45-110` owns the separate rs-ui Tree root, retains TreeItems with expanded/selected/focus state, routes activation through `BehaviorCommand::Activate`, prunes stale subtrees, and dispatches queued focus traversal.
- At that SHA, `crates/ui/src/native_explorer_tree_runtime.rs:138-180` synchronizes semantics and hit-test bounds and removes stale subtrees; `crates/ui/src/explorer_tree.rs:31-58` builds length-prefixed stable keys; `:60-91` maps egui ArrowUp/ArrowDown and row activation into rs-ui behavior while retaining logged egui fallback on adapter errors.
- Explorer connection, database, schema, Tables folder, and table views register rows into the hierarchy and keep the existing DB Pro reducer/action paths. The TreeItems cover the rendered hierarchy in this slice; other schema object folders/rows are not yet registered.
- Keyboard focus navigation is tested over registered items, not in the native app. Since only rows in the current visible viewport register with rs-ui, focus traversal past clipped/unrendered rows is not proven and remains a known limitation.

## Automated checks

| Command | Result |
|---|---|
| `cargo test -p db-pro-ui native_runtime_shell::tests -- --nocapture` | PASS — 9 passed / 0 failed / 0 ignored, exit 0; includes tab/close semantics, normalized activation, stale-node cleanup, and both output splitter directions. |
| `cargo test -p db-pro-ui --quiet` | PASS — 950 passed / 0 failed / 0 ignored, exit 0 |
| `cargo check --workspace` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo test --workspace --quiet` | PASS — 1,607 passed / 0 failed / 41 ignored, exit 0 |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `git diff --check` | PASS, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::tests --no-fail-fast` | PASS — 5 passed / 0 failed / 0 ignored, exit 0 |
| `cargo fmt --all -- --check` | FAIL/BLOCKED — Cargo metadata cannot load `rs-ui-core` because `/data/dev/projects/ui-runtime-foundation/crates/ui-core/Cargo.toml` is missing. |
| `cargo test -p db-pro-ui result_grid_virtual_adapter --lib` | FAIL/BLOCKED — same missing `rs-ui-core` local path dependency before compilation. |
| `rustfmt --check crates/ui/src/result_grid_virtual_adapter.rs crates/ui/benches/result_grid_benchmarks.rs` | PASS, exit 0. |
| `git diff --check` | PASS, exit 0. |
| `cargo check --workspace` | FAIL/BLOCKED — same missing `rs-ui-core` local path dependency before compilation. |
| `cargo clippy --workspace --all-targets -- -D warnings` | FAIL/BLOCKED — same missing `rs-ui-core` local path dependency before compilation. |
| `cargo test --workspace` | FAIL/BLOCKED — same missing `rs-ui-core` local path dependency before compilation. |
| `cargo build --release --locked -p db-pro-native` | FAIL/BLOCKED — same missing `rs-ui-core` local path dependency before compilation. |
| `cargo test -p db-pro-ui sidebar_resize_routes_pointer_delta_through_rs_ui_runtime -- --nocapture` | PASS — 1 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui navigator_tree_scrolls_with_the_mouse_wheel -- --nocapture` | PASS — 1 passed / 0 failed / 0 ignored; verifies egui/rs-ui offsets match |
| `cargo check --workspace` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS against the final adapter source, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::tests --quiet` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS — 12 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui explorer_tree::tests --quiet` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS — 1 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui --quiet` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS — 954 passed / 0 failed / 0 ignored, exit 0 |
| `cargo check -p db-pro-ui` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS — 14 pass / 2 pre-existing heuristic warnings / 0 fail |
| `git diff --check` at `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `cargo fmt --all -- --check` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS, exit 0 |
| `cargo check --workspace` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS, exit 0 |
| `cargo test --workspace --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 1,611 passed / 0 failed / 41 ignored, exit 0 |
| `cargo build --release --locked -p db-pro-native` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::native_explorer_tree_runtime::tests --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 3 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::tests --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 9 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui explorer_tree::tests --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 1 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 954 passed / 0 failed / 0 ignored, exit 0 |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` after the runtime extraction | PASS — 15 pass / 1 warning category / 0 fail; the new helper warning is the four-component stable table identity key |
| `cargo fmt --all -- --check` at source SHA `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `cargo check --workspace` at source SHA `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` at source SHA `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `cargo test --workspace --quiet` at source SHA `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS — 1,611 passed / 0 failed / 41 ignored, exit 0 |
| `cargo build --release --locked -p db-pro-native` at source SHA `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7` | PASS, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::native_explorer_tree_runtime::tests --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 3 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui native_runtime_shell::tests --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 9 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui explorer_tree::tests --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 1 passed / 0 failed / 0 ignored, exit 0 |
| `cargo test -p db-pro-ui --quiet` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 954 passed / 0 failed / 0 ignored, exit 0 |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` at source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8` | PASS — 15 pass / 1 warning category / 0 fail; `table_tree_key` uses four identity components; two reports are existing painting helpers |
| `cargo test --workspace --quiet` | PASS — 1,603 passed / 0 failed / 41 ignored, exit 0 |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 14 pass / 2 warnings / 0 fail. Both warnings are existing long/parameter-heavy functions in `workspace_tabs_surface_view.rs`; no new adapter warning. |
| `git diff --check` | PASS, exit 0 |
| `cargo build --release --locked -p db-pro-native --features capture` | NOT RUN in this continuation; it passed during the prior shell-layout pass. |

### Stage 4 current continuation — 2026-10-02

| Command / evidence | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS, exit 0, after adapter and benchmark edits. |
| `git diff --check` | PASS, exit 0. |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 15 pass / 1 heuristic warning / 0 fail. The warning is `draw_result_grid` (78 lines); the same function is 79 lines at baseline `5c1f42bad87c69f3cbfddcb921c557987706201e`, so it is not introduced by Stage 4. |
| `cargo test -p db-pro-ui result_grid_virtual_adapter::tests --no-fail-fast` | FAIL before tests — 30 missing rs-ui behavior/resize API errors already used by Stages 1–3; adapter tests did not execute. |
| `cargo check --workspace` | FAIL — same 30 missing rs-ui API errors. |
| `cargo clippy --workspace --all-targets -- -D warnings` | FAIL — same compile errors; Clippy did not run on the crate. |
| `cargo test --workspace` | FAIL before tests — same compile errors. |
| `cargo build --release --locked -p db-pro-native` | FAIL — same compile errors; no release binary produced. |
| `cargo bench -p db-pro-ui --bench result_grid_benchmarks -- --test` | FAIL before benchmark execution — same compile errors. |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | FAIL — release build and Clippy fail on the API mismatch; 3 checks failed, 4 were not run; no performance artifact or measurement produced. |
| Native result-grid runtime capture / 1280×800, 1440×900, 1920×1080 | NOT RUN; the package does not compile against the available rs-ui checkout. |
| Benchmark coverage added | Retained `VirtualGrid` prepare/window cases at 10k, 100k and 1M rows with 50 fixed-width columns. No before/after timing, allocation, or frame-preparation result is available. |

#### Stage 4A ownership trace

| Concern | Current source of truth / behavior |
|---|---|
| Row count | `UiQueryResult` owns returned rows; `GridProjectionCache` provides filtered/sorted source-row indexes; displayed count is `indexes.len()`. |
| Visible rows | Prior implementation: egui `ScrollArea::vertical().show_rows`. Working-tree adapter: rs-ui `ScrollState` / retained `VirtualGrid` calculates row range plus one-row overscan from egui's vertical scroll offset. egui remains scroll input/offset owner. |
| Column widths / order | DB Pro `TableDataState` owns persisted widths and order; defaults 180 px, width state clamps 60–520 px. |
| Horizontal scroll / columns | egui `ScrollArea::horizontal` owns scroll; all ordered columns are still painted because rs-ui's current grid API is fixed-width only. |
| Selection / keyboard | DB Pro `TableDataState`, grid interaction handlers, and `handle_grid_keyboard` own cell/row/range selection, navigation and clipboard. No rs-ui selection model is present in the available checkout. |
| Resize | Existing DB Pro grid/header interaction and persisted width state remain authoritative; no rs-ui grid `Resizable` API is present. |
| Typed cells / formatting | `UiCell` remains typed through projection and visible row paint; formatting occurs in the per-cell painter. No dataset-wide stringify or row clone was added. |
| Paint path | `UiQueryResult` → `GridProjectionCache` → filtered/sorted indexes → adapter visible row range → `GridRows` → egui row/cell painter. |

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

At source SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8`, the clean-code scan reports one parameter-count warning category. One new helper, `table_tree_key`, takes the four identity components that define a stable tree path; the other two reports are existing Explorer row-paint helpers.

## Remaining limitations

- Sidebar and query-output splitter resize use rs-ui `Resizable`; egui remains responsible for pointer capture and painting. Tab selection activation goes through rs-ui `Pressable`, while DB Pro retains the existing action dispatch.
- Native keyboard focus/accessibility behavior remains unverified. Native app smoke and 1920×1080 capture are still outstanding.
- Result-grid vertical virtualization has a working-tree adapter but no passing integration test or runtime evidence yet. Horizontal virtualization, rs-ui selection/resize routing, and search/filter migration remain incomplete. Explorer connection/database/schema/Tables/table rows now have source-level TreeItem adapters, but other schema-object folders/rows are not represented.
- The rs-ui semantic tree remains internal to the UI adapter; native OS accessibility output for Explorer has not been verified.
- The local sibling rs-ui source is clean at `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`, but is API-incompatible with current Stages 1–3. No compatible immutable dependency pin is available in the local refs.
- Full rs-ui renderer/window integration is not implemented; DB Pro remains on eframe/egui glow.
- 1920×1080 capture and native end-to-end smoke remain pending.
