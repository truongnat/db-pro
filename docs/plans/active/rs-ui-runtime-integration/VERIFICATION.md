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
- Historical Stage 4 adapter finding: the vertical range adapter left egui as host/painter and DB Pro owned projection, typed cells, widths, and selection. The current exact pin and Stage 4.2 implementation are recorded below.

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

### Historical Stage 4 continuation before compatible rs-ui pin — 2026-10-02

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
| Selection / keyboard | DB Pro `TableDataState` owns selected cell/row/rows and anchors. rs-ui `SelectionModel` calculates projected-key single/toggle/range operations; existing keyboard navigation and clipboard remain DB Pro-owned. |
| Resize | rs-ui `Resizable` calculates result-column pointer movement and clamped width. DB Pro `TableDataState` owns and persists the returned width. |
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
- Result-grid vertical virtualization has focused tests and benchmark evidence, but no native runtime capture yet. Horizontal virtualization and search/filter migration remain incomplete. Explorer connection/database/schema/Tables/table rows now have source-level TreeItem adapters, but other schema-object folders/rows are not represented.
- The rs-ui semantic tree remains internal to the UI adapter; native OS accessibility output for Explorer has not been verified.
- Historical only: rs-ui `db3cf2bfed3d29f0e1d19963462488ed9157f1ea` is API-incompatible with Stages 1–3. Current Cargo dependencies and lockfile pin `f6e798d6cfa966b5344cf6a9de6c634563258eec`.
- Full rs-ui renderer/window integration is not implemented; DB Pro remains on eframe/egui glow.
- 1920×1080 capture and native end-to-end smoke remain pending.

### Current continuation — exact pin and Stage 4.2 — 2026-10-02

Source under verification: DB Pro base commit `00a077e837fced6104bd34003340440847fe2f2a` plus the uncommitted changes in this working tree. rs-ui dependency revision: `f6e798d6cfa966b5344cf6a9de6c634563258eec`.

| Gate | Result |
|---|---|
| `cargo check -p db-pro-ui` | PASS |
| `cargo check --workspace` | PASS |
| Stage 1–3 focused shell/Explorer/resize/scroll tests | PASS: 15 tests across the requested commands |
| `cargo test -p db-pro-ui result_grid_virtual_adapter --no-fail-fast` | PASS: 4 tests |
| Stage 4.2 SelectionModel single/toggle/range tests | PASS: 2 focused tests |
| Stage 4.2 Resizable integration test | PASS: 1 focused test |
| `cargo fmt --all -- --check` | PASS after Stage 4.2 changes. |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS after Stage 4.2 changes. |
| `cargo test --workspace --no-fail-fast` | PASS: 961 passed, 0 failed. |
| `cargo build --release --locked -p db-pro-native` | PASS after Stage 4.2 changes. |
| `git diff --check` | PASS after Stage 4.2 changes and plan updates. |

Cargo.lock records `git+https://github.com/truongnat/rs-ui?rev=f6e798d6cfa966b5344cf6a9de6c634563258eec#f6e798d6cfa966b5344cf6a9de6c634563258eec` for `ui-core`, `ui-runtime`, and `ui-text`. Release build compiled `ui-core` and `ui-runtime` from that Git source.

Perf scan: WARN, 3 passed / 1 warning / 0 failed; binary is 51.4 MB against the 50 MB target threshold (same rounded size as the pre-Stage-4.2 scan, so no size change is observed at scan precision). The scan could not initially locate the artifact because Cargo uses `/data/cargo-target`; a temporary symlink allowed artifact measurement and was removed afterward. No CI files were changed. Native/backend/DB runtime portions were not run by the scan.

Criterion `result_grid_benchmarks` PASS (no comparable before baseline): fixed-width `VirtualGrid` set-scroll plus visible-cell materialization measured 10k rows at 1.4926–1.5049 µs, 100k at 1.6940–1.7063 µs, and 1M at 1.8787–1.8969 µs. With 560 px viewport, 28 px rows, 600 px width, 120 px columns and overscan 1, this benchmark produces 22 rows × 6 columns = 132 virtual cells. DB Pro currently paints all 50 ordered columns for those 22 rows (about 1,100 cells); column virtualization remains blocked on variable-width virtual-axis support.

Stage 4.2 maps current DB Pro row/cell selection operations through rs-ui `SelectionModel` using projected source-row keys, then stores results in `TableDataState`. Column drag maps cumulative pointer delta to rs-ui `Resizable`; returned widths are stored only in DB Pro state. Clipboard and sort/filter behavior are unchanged. Native result-grid interaction, accessibility, and 1920×1080 evidence remain unverified.
