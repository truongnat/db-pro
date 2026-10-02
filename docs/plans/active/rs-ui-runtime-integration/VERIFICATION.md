# Verification — rs-ui Runtime Integration

## Current Explorer paint verification — 2026-10-02

Source: DB Pro base `977a5dd00e656e3529e18372edbc121496cb700e`, exact local source tree `e317464c083188b75c04d5be1e1a9f4517086b38`, canonical rs-ui `727d65d3957eb7bbfbd316e68a272ca123ab411c`. The tree was written through an isolated Git index over Cargo.lock, crates/ui and crates/native-app; it is not a published DB Pro commit. Prior Welcome spike WIP remains included, separate from Explorer scope. Historical results below apply to their stated revisions.

| Gate | Executed result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS: 1,619 passed, 0 failed, 43 ignored |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `cargo build --release --locked -p db-pro-native --features capture` | PASS |
| Stage 1–3 focused runtime shell / Explorer semantics | PASS: 13 runtime tests, 1 Explorer test, separate resize and wheel regression tests |
| `native_scroll_preserves_persisted_tree_namespace` | PASS: custom viewport preserves existing collapse IDs |
| `explorer_renderer_preserves_row_activation_expand_and_focus -- --ignored` | PASS: explicit GPU row activation/chevron/focus comparison against egui |
| `explorer_renderer_runtime_benchmark --release -- --ignored --nocapture` | PASS: explicit GPU benchmark, wheel offset and actual scrollbar thumb drag assertions |
| Clean-code scan `rust --diff --ratchet --ci` | PASS: 14 checks, 2 reviewed heuristic warnings, 0 failures; does not constitute independent review |
| Performance scan | WARN: default binary 62.5 MiB exceeds 50 MiB target (below 100 MiB critical); no performance improvement claim |
| `git diff --check` | PASS |

Logs, runnable capture script and capture binary are outside Git at `/home/vietis/.agents/outputs/db-pro/testresults/explorer-renderer/`. The perf script assumes repo-local target; a temporary ignored symlink to configured `/data/cargo-target` was used and removed. The first scan without that target mapping failed and is retained as evidence.

### Runtime measurements

Ten measured warm frames per mode, three warm-up frames; synthetic 1k/10k connection roots on Intel UHD Graphics 630 / Vulkan. Measurement includes UI preparation and changed-scene canonical GPU render/readback, excludes actual Glow texture upload/presentation. No statistical significance or end-to-end improvement is asserted.

| Nodes | Painter | Idle preparation | Scroll preparation including readback |
|---|---|---|---|
| 1,000 | egui | 3.170 ms | 3.003 ms |
| 1,000 | rs-ui | 1.523 ms | 4.931 ms |
| 10,000 | egui | 35.201 ms | 35.378 ms |
| 10,000 | rs-ui | 25.202 ms | 28.900 ms |

These measurements precede only the final shared-constant import cleanup and capture-only fixture persistence fix; production geometry/behavior is identical. Both 10k paths exceed a 16 ms frame budget. Readback is a temporary bridge limitation.

### Native visual evidence and limitations

Capture matrix completed: 48 raw native captures and 24 comparison sheets, normal/loading/error/empty at logical 1280×800, 1440×900, 1920×1080, scales 1× and 2×. Physical framebuffer sizes are respectively logical size and twice each dimension. Capture-only fixtures are reapplied after update so asynchronous startup catalog events cannot replace error/loading with empty. Actual hardware renderer: Intel UHD Graphics 630 (CML GT2), Vulkan, Mesa 26.0.8-1ubuntu0.3; Xvfb hosts the native window. This is simulated HiDPI, not a physical Retina test.

Artifacts: `/home/vietis/.agents/outputs/db-pro/artifacts/explorer-renderer/`; matched native-pixel comparison names include `compare-normal-1280x800-1x.png`, `compare-normal-1440x900-2x.png`, `compare-error-1280x800-1x.png`, `compare-empty-1280x800-1x.png` (egui left, rs-ui right). Sidebar chrome/search/context menus remain egui. Only Explorer content, row fills/text/Lucide/indent/chevrons and scroll viewport use canonical rs-ui rendering. Provider calls are not exercised; PostgreSQL/SQLite business logic is unchanged.

Visual inspection: placement is clean at 1×/2×, but rs-ui small text remains visibly lighter than egui. Error text wraps without clipping; empty-state button styling differs. **Visual quality ≥ current egui has not been accepted.** Native keyboard traversal beyond registered visible rows remains an existing limitation. Current slice stays `RUNTIME_VERIFY`, overall integration `IMPLEMENTING`; no further surface cutover is authorized by this result.

Binary SHA-256: capture `41198988f838c264b9d0ba41241a2b5530b19e86f53864010943e7d21b61c86e`; default release `1c0dc332371dc3f3a40f8ce9c74ba6336121c1ab4f8e3a6d5076e7ae85e9ae89` (65,575,360 bytes).

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

### Current continuation — committed Git pin and native Result Grid attempt — 2026-10-02

Source under verification: DB Pro `433afec96a24da4ec801b02ae31d3a91a749f949`; rs-ui `f6e798d6cfa966b5344cf6a9de6c634563258eec`. The untracked duplicate `crates/ui-runtime/src/selection.rs` was removed from the local checkout; it was not tracked by Git. Working tree is clean. Earlier failures below that cite a missing local `rs-ui-core` path are historical and superseded by the exact Git pin.

| Gate | Result |
|---|---|
| `cargo check -p db-pro-ui` | PASS |
| `cargo check --workspace` | PASS |
| Stage 1–3 focused shell/Explorer/resize/scroll tests | PASS: 16 tests across requested commands |
| `cargo test -p db-pro-ui result_grid_virtual_adapter --no-fail-fast` | PASS: 4 tests |
| `cargo test -p db-pro-ui app::table_data_state::tests --no-fail-fast` | PASS: 9 tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace --no-fail-fast` | PASS: 961 passed, 0 failed |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `git diff --check` | PASS |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | WARN: 3 passed / 1 warning / 0 failed; 51.4 MB binary vs 50 MB target; runtime-specific scan checks were not run |
| `cargo bench -p db-pro-ui --bench result_grid_benchmarks` | PASS; latest fixed-width prepare medians: 10k rows 1.4851 µs, 100k 1.5492 µs, 1M 1.7680 µs. No comparable before baseline; no improvement claim. |

Native app launch connected to saved local PostgreSQL and completed schema introspection (`tables=1110`). The UI remained on Welcome; synthetic mouse input did not activate the query tab/editor, and no SQL was submitted. Result Grid rendering, selection, resize, keyboard interaction, and accessibility therefore remain runtime-unverified. Screenshot captures are temporary under `/tmp`; no 1920×1080 artifact was produced. Stage 4 remains in verification, not complete.

### Welcome renderer spike — 2026-10-02

Scope: standalone, optional `rs-ui-welcome-spike` binary. Production Welcome and all other surfaces continue using their existing painter. Base DB Pro SHA `977a5dd00e656e3529e18372edbc121496cb700e` plus the uncommitted spike diff; all five rs-ui crates resolve exact Git SHA `f6e798d6cfa966b5344cf6a9de6c634563258eec`. Cargo.lock adds 55 package versions and removes/upgrades none of the existing package versions.

Run:

```bash
DB_PRO_WINDOW_SIZE=1280x800 cargo run --locked -p db-pro-native --features rs-ui-spike --bin rs-ui-welcome-spike
```

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS: 1,618 passed / 0 failed / 41 ignored across 22 test-result summaries |
| `cargo check -p db-pro-native --features rs-ui-spike --all-targets` | PASS |
| `cargo clippy -p db-pro-native --features rs-ui-spike --all-targets -- -D warnings` | PASS |
| `cargo test -p db-pro-native --features rs-ui-spike` | PASS: 31 existing native-app tests; standalone spike has no unit tests |
| `cargo build --release --locked -p db-pro-native` | PASS; default feature set excludes the spike |
| `cargo build --release --locked -p db-pro-native --features capture --bin db-pro-native` | PASS; existing framebuffer capture driver used for egui evidence |
| `cargo build -p db-pro-native --features capture,rs-ui-spike --bins` | PASS; final spike rebuilt with `--features rs-ui-spike --bin rs-ui-welcome-spike` |
| `git diff --check` | PASS |
| Clean-code scan, changed files, ratchet/CI mode | 14 pass / 2 warning categories / 0 failure: four-argument text helper and linear scene/GPU constructors; no swallowed errors or debug printlns |
| Performance scan | WARN: 51.4 MB default production binary exceeds 50 MB target; 3 pass / 1 warning / 0 failure. Benchmark/provider-runtime scan sections not executed; no performance claim. |

Logs and reproduction scripts: `/home/vietis/.agents/outputs/db-pro/testresults/rs-ui-welcome-spike/`. Exact image hashes and dimensions: `/home/vietis/.agents/outputs/db-pro/artifacts/rs-ui-welcome-spike/capture-manifest.json`. Raw screenshots, full comparison sheets and detail crops are in that artifact directory. Comparison sheets copy native pixels; no sharpening or resampling is applied.

Final performance-scan provenance: source `977a5dd00e656e3529e18372edbc121496cb700e+dirty(11)` (including a temporary `target` symlink used to locate Cargo's shared target directory); default production binary SHA256 `dd74888555250d4c30b758aad851ee1aef9002945983450c44bcbe20ee864688`, 51.4 MB. The symlink was removed afterward. Native window captures are serialized and wait for presentation to settle; an earlier concurrent capture pass produced occluded black regions on compositor-free Xvfb and was discarded/replaced. Final PNGs pass dimension, background and foreground checks and were visually inspected at native pixel size.

| Logical viewport | Scale | Physical framebuffer, both renderers | egui / spike native run |
|---|---:|---|---|
| 1280×800 | 1 | 1280×800 | PASS / PASS |
| 1440×900 | 1 | 1440×900 | PASS / PASS |
| 1920×1080 | 1 | 1920×1080 | PASS / PASS |
| 1280×800 | 2 | 2560×1600 | PASS / PASS |
| 1440×900 | 2 | 2880×1800 | PASS / PASS |
| 1920×1080 | 2 | 3840×2160 | PASS / PASS |

Native runs used Xvfb/X11 with `WINIT_X11_SCALE_FACTOR=1/2`, not a Retina monitor. Spike reported Vulkan, `llvmpipe (LLVM 21.1.8, 256 bits)`, `Bgra8UnormSrgb`, opaque surface composition and default MSAA 4×; rs-ui pipelines use premultiplied blending. Each spike emitted 26 display commands, with glyph rasterization and atlas usage recorded in its log. Physical screenshot dimensions were independently asserted. Escape closed each spike normally (exit 0). Baseline screenshots use the release application and an isolated empty data/config directory, avoiding the debug build's developer connection preset. No SQL or database flow is part of the spike. Loading/error/provider states are n/a for this static, disconnected visual experiment.

Text path is `TextSystem::shape` (cosmic-text 0.19) → `UiTree::set_text` → `UiTree::paint` → `UiRenderer::render` → swash / rs-ui glyph atlas → wgpu. Host geometry stays logical; `Viewport` receives actual physical size/scale. Canonical rs-ui snaps shape edges and retains text positioning; DB Pro does not implement glyph rasterization, snapping, atlas uploads, shaders, or a compatibility renderer.

Visual verdict: **clearer-than-egui NOT DEMONSTRATED**. The renderer presents native Welcome shapes successfully at both scales. However egui uses bundled Inter/Inter Medium while rs-ui's generic family resolves through system fonts, and square/cross probes are not the original Lucide icons. A standalone text probe at the same Git revision reproduces Lato letters plus Noto Color Emoji for the space in `DB Pro`, whose advance is 39.84375 px at size 32. Button widths consume actual rs-ui text metrics so labels are not clipped, but excessive word gaps remain visible. Font-policy/fallback needs a canonical rs-ui follow-up and a matched-font repeat of this Welcome experiment before any production cutover.

Source SHA256: host `d1c29a035a1e0b1c105fe80349fef099b5bf9671cdb2232d6f2fbf16d0e23b1d`; scene `20696d54c7fa253a8b1d08d3c2376a81a35a14b8ddf7044b1f179b3f6a68b5e2`. Captured spike debug binary SHA256 `4d9faa2eab4ad6b0024cd8eda07697d15a2fa2bbf143dfc4f02a6035b4717dab`; egui release capture binary `a33281078a752944b9240ea0cb643e81743f261aad46082169c7c426c82ea660`. Self-review only; this experiment does not complete the integration plan or advance Stage 5.
