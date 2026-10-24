---
schema_version: 1
id: "MEM-PLAN-rs-ui-runtime-integration-b99ecd3e9f7e"
title: "Plan archive: Rs Ui Runtime Integration"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, rs-ui-runtime-integration]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b99ecd3e9f7e4d07152f2f454559bcd7f2e126d638741104a829100a301642c2"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/rs-ui-runtime-integration/PLAN.md"
  - "docs/plans/active/rs-ui-runtime-integration/FINDINGS.md"
  - "docs/plans/active/rs-ui-runtime-integration/VERIFICATION.md"
  - "docs/plans/active/rs-ui-runtime-integration/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/rs-ui-runtime-integration/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# rs-ui Runtime Integration

State: BACKLOG — deferred by owner on 2026-10-02
Branch: `main` (repository owner workflow override)

## Owner decision (2026-10-02)

All executable rs-ui integration is removed from DB Pro in favor of the existing egui UI. The independent rs-ui repository is retained. Prior implementation and verification records below are historical, not the shipping state. Restoration evidence: `../restore-egui-ui/`.

## Goal

Adopt the separate rs-ui runtime incrementally as the UI layout and behavior foundation for DB Pro while preserving the existing `DbProApp`, task bridge, domain state, database services, and egui window host.

## Scope

- Owner-authorized Explorer-only paint cutover: the existing tree hitboxes and
  state/actions feed a canonical rs-ui display list, shaped with DB Pro's
  bundled Inter/Lucide fonts, rendered by wgpu into a cached texture for the
  Glow host. Only the Explorer tree viewport migrates; sidebar chrome/search
  inputs and context menus remain host surfaces. The other application paint
  paths and provider logic stay outside this change.

- Pin the rs-ui core and runtime crates to an exact Git revision.
- Create a thin DB Pro shell layout adapter backed by `UiTree`.
- Use rs-ui-computed sidebar geometry in the existing egui host.
- Route sidebar resizing and focused keyboard adjustment through rs-ui behavior
  while DB Pro remains the source of persisted panel width.
- Route sidebar wheel input and retained offset through rs-ui `ScrollState`,
  keeping egui for content painting and scrollbar interaction during migration.
- Mirror the visible Explorer connection/database/schema/Tables/table hierarchy
  into rs-ui `Tree` / `TreeItem` semantics with stable model-derived keys,
  selection, expansion, focus and keyboard traversal while preserving existing
  DB Pro actions and schema state.
- Keep the renderer boundary explicit and track the next migration stages.
- Run a Welcome-only renderer cutover spike in an optional standalone binary:
  DB Pro Welcome snapshot → `UiTree` → `DisplayList` → `UiRenderer` → wgpu → `UiWindow`.
  Compare the current egui Welcome with the spike at 1280×800 and 1440×900;
  record logical/physical dimensions, scale, font differences, and visual limits.
  This experiment does not advance text-input or production renderer migration.

## Non-goals

- Replacing the eframe window host or application-wide renderer.
- Rewriting product state, commands, connection/query/schema logic, or editor.
- Rewriting the result-grid painter or moving database/projection logic into rs-ui.
- Migrating search/filter inputs or the SQL editor in this stage.
- Copying rs-ui source into DB Pro.

## Current architecture and boundary

DB Pro's shipped UI is `db-pro-ui` (`crates/ui`) hosted by `db-pro-native` (`crates/native-app`) through eframe/egui. `DbProApp` owns product UI state and communicates through the existing typed command/event bridge. The DB Pro runtime and domain crates remain independent of UI rendering.

The rs-ui workspace provides `ui-core` geometry/display-list types and `ui-runtime` layout, interaction, focus, scroll, and accessibility behavior. Its renderer is a separate wgpu renderer and its window crate uses winit. This first adapter consumes only the renderer-independent core/runtime boundary; egui remains the host and painter while migration is incremental.

## Reuse and coupling

- Reuse `DbProApp`, existing workspace/connection/query/schema state, command/event handling, persistence, and provider services unchanged.
- Keep rs-ui types inside `crates/ui` adapters. Do not add rs-ui dependencies to core, infrastructure, or runtime.
- Keep current tab actions and sidebar content implementations; adapters return
  into existing DB Pro state/action handling.

## Migration order

1. Host integration and shell geometry adapter.
2. Sidebar/split/tab geometry adapter, then normalized input/action routing.
3. Schema tree adapter.
4. Virtual result grid adapter with typed-cell preservation and measurements.
5. Search/filter inputs.
6. Decide separately whether SQL editor migration is warranted.
7. Remove egui shell pieces only after runtime and visual evidence.

## Risks

- Historical blocker: rs-ui `db3cf2bfed3d29f0e1d19963462488ed9157f1ea` lacked behavior/resize APIs used by DB Pro. That finding applies only to that SHA. The current exact pin is `727d65d3957eb7bbfbd316e68a272ca123ab411c`.
- eframe currently uses the glow renderer; rs-ui's renderer targets wgpu. Switching rendering backends is outside this adapter and needs an isolated technical spike before any full renderer migration.
- The current host adapter rebuilds a small retained tree for the shell layout request. Measure before expanding that pattern to high-frequency surfaces.
- Before the Explorer paint continuation, rows used egui for painting, expansion state, and row
  viewport clipping. The adapter mirrors visible rows into rs-ui and routes
  ArrowUp/ArrowDown focus traversal through its behavior runtime; it does not
  replace Explorer rendering or expose rs-ui semantics through a native OS
  accessibility backend. Keyboard traversal can only target rows rendered in
  the current viewport.
- The Explorer continuation uses existing expansion/selection/action owners,
  rs-ui text/renderer and canonical scroll offset. Changed scenes incur GPU
  readback and Glow host texture upload; unchanged scenes reuse the texture.
  Native primary backends avoid a GLES/EGL conflict with the current Glow
  context. Vulkan, Metal or DX12 is required. Visual text weight differs from
  egui's gamma-space blend; screenshot acceptance remains explicit.
- Stage 4 uses rs-ui `VirtualGrid` and `ScrollState` to calculate the vertical
  row window from egui's scroll offset while egui remains the scroll host and
  painter. DB Pro retains filtered projection, typed cells, column order,
  variable widths, selection, and persisted width state. Horizontal
  virtualization is deferred because the pinned `VirtualGrid` API only accepts
  one fixed column extent. SelectionModel and Resizable now handle behavior;
  DB Pro remains the source of truth for selected rows/cells and widths.
- Stage 4 source/tests and build gates pass against the exact Git pin. Native
  result-grid runtime evidence remains outstanding.

## Acceptance

- The DB Pro UI crate consumes canonical rs-ui core/runtime/text/renderer at
  exact Git revision `727d65d3957eb7bbfbd316e68a272ca123ab411c`; it does not
  implement its own renderer or text rasterizer.
- The shell adapter uses rs-ui layout results and rejects viewports that cannot fit a usable main surface.
- Existing domain state and command/task bridge remain unchanged.
- Explorer semantics represent the existing visible hierarchy and clicks
  continue through DB Pro's existing selection and action paths.
- Stage 4 must preserve the existing projection, typed cells, column widths,
  selection, clipboard, and egui cell painter while measuring visible work.
- Targeted tests and Rust checks are recorded honestly.
- Runtime screenshots and end-to-end smoke evidence remain required before marking this integration complete.


## Source: `FINDINGS.md`

# Findings — rs-ui Runtime Integration

## Explorer-only paint continuation — 2026-10-02

- DB Pro base is `977a5dd00e656e3529e18372edbc121496cb700e`; exact local source snapshot is Git tree `e317464c083188b75c04d5be1e1a9f4517086b38` (isolated index covering Cargo.lock, crates/ui and crates/native-app; not a published commit).
- Canonical rs-ui `727d65d3957eb7bbfbd316e68a272ca123ab411c` exposes `TextSystem::with_fonts` and `FontFamily::Named`; 95 tests pass, one diagnostic remains ignored, and fmt/check/clippy pass. The commit was pushed to origin/main and DB Pro consumes exact Git source.
- Font family metadata is `Inter`, `Inter Medium`, and `lucide`. The adapter reuses DbProTheme font bytes/colors. Cosmic-text → swash → canonical atlas/UiRenderer remains the text pipeline; no renderer is copied into DB Pro.
- Eframe 0.29's Glow host and canonical wgpu 30 cannot share the existing device integration. Changed scenes use an opaque sRGB offscreen framebuffer/readback/host texture upload; identical scene/size/scale reuses the texture. The cache retains visible text runs and no authoritative domain, selection or expansion state.
- Native startup exposed a GLES/EGL `BadAccess` panic during adapter enumeration with Glow current. Native primary backends fix this conflict. Captures use Intel UHD Graphics 630 / Vulkan / Mesa 26.0.8. Simulated 2× captures do not establish physical Retina behavior.
- Canonical ScrollState owns Explorer wheel/thumb/track offset and content bounds. The migrated viewport stores no egui ScrollArea offset. Other sidebar activities retain their host path.
- The custom viewport initially changed persisted collapse IDs. Keeping the original ScrollArea child namespace fixes this; a regression test compares the two namespaces. Real-GPU checks cover wheel/thumb drag and equivalent row activation/expand/focus.
- Startup catalog events could overwrite a one-shot capture fixture. The capture-only harness reapplies the Explorer fixture after each update; production state/event handling is unchanged. Capture completion alone does not prove the requested state was visible.
- Initial native error detail clipped content. A width-constrained canonical monospace run now preserves the full error and tooltip, while the existing refresh action remains unchanged.
- Matched-font captures show sharp placement but lighter text than egui. The canonical renderer blends coverage in linear light into sRGB; egui_glow multiplies colors in gamma space. This distinction does not prove visual superiority; sign-off remains pending.
- Sidebar chrome/search/context menus remain host surfaces. Topbar, tabs, editor, result grid, status bar and provider logic do not migrate. The pre-existing Welcome spike remains separate WIP.

## Welcome renderer spike — source audit, 2026-10-02

- DB Pro baseline `977a5dd00e656e3529e18372edbc121496cb700e`; canonical rs-ui `f6e798d6cfa966b5344cf6a9de6c634563258eec`.
- Native-app's optional `rs-ui-spike` feature activates the five rs-ui crates, wgpu 30 and a standalone native binary. Production default binary and Glow host stay separate.
- `UiTree::paint` emits the display list; `UiRenderer` owns rendering, physical-pixel shape snapping, sRGB output and premultiplied blending. Its default uses MSAA 4×. Text is shaped by cosmic-text and rasterized by swash into rs-ui's glyph atlas. The host passes window physical size and scale without rounding text origins.
- Welcome uses existing `DbProTheme` semantic colors and a disconnected state snapshot with the DB Pro `UiConnectionSummary` type. Buttons are visual probes; no business actions or database worker is attached. Empty space reserves the current production shell footprint without painting other surfaces.
- Text comparison has a font confounder: production egui uses bundled Inter/Inter Medium; the pinned `ui-text` API offers generic Sans/Serif/Monospace only and obtains fonts from the system. A renderer sharpness claim needs this limitation stated.
- Six native runs per renderer completed at 1280×800, 1440×900 and 1920×1080 logical points, with scale 1× and 2×. wgpu selected Vulkan/llvmpipe on Xvfb. Screenshots and native-pixel comparison sheets live outside Git under `/home/vietis/.agents/outputs/db-pro/artifacts/rs-ui-welcome-spike/`.
- Visual verdict: sharper-than-egui is **not demonstrated**. Rectangles and borders render cleanly, but the fonts and icon geometry differ; no controlled improvement claim is justified.
- Confirmed text fallback issue at the pinned rs-ui revision on this Linux font set: an independent `TextSystem`/cosmic-text probe resolves letters in `DB Pro` to Lato and the ASCII space to Noto Color Emoji. The space advances 39.84375 logical px at font size 32; `Open in editor` measures 108.77405 px at size 13. This reproduces the excessive word gaps without DB Pro, `UiTree`, or a renderer. Spike button widths now consume actual text metrics to prevent clipping.
- Canonical font configuration/fallback belongs in rs-ui. Do not patch strings, shape glyphs, or implement a renderer/font-selection shim in DB Pro. Repeat the same Welcome comparison with matched fonts before another production cutover. Start/Connections cards are the next small candidate after that gate passes.

Baseline: `db-pro@c0c1f5525b20a810913d1eee13c7ee2dd15b6664` before the original integration. Current Stage 3 implementation/refactor: `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8`; initial Stage 3 source commit: `6044bf07a2e30c2fa4dfa03f92315c2114d63ec7`; Stage 2 source: `42f14e520fa1e8bb280c1ec0399d3f523d4792da`. The available rs-ui checkout is clean at `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`.

## Evidence

- DB Pro UI entry is `DbProApp::update` and calls the existing shell lifecycle in `crates/ui/src/app_lifecycle.rs`.
- The sidebar is hosted by egui `SidePanel` in `crates/ui/src/sidebar_surface_view.rs`.
- DB Pro's service and domain code are behind the existing UI command/event bridge, documented in `docs/architecture/system-overview.md`.
- rs-ui's workspace separates `ui-core`, `ui-runtime`, `ui-renderer`, and `ui-window`; runtime owns retained layout and behavior while the renderer uses wgpu.
- At rs-ui SHA `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`, `VirtualGrid::new_fixed` and `ScrollState` are public, but the checkout does not define `BehaviorCommand`, `ResizeAxis`, `ResizeConfig`, `SelectionModel`, or `Resizable`, nor `UiTree::dispatch_behavior_command` / `register_resizable`. DB Pro Stages 1–3 already reference the missing behavior/resize APIs.
- The sidebar resize handle uses egui for pointer capture/focus detection, then calls rs-ui `Resizable` APIs for pointer delta, bounds, keyboard movement, and slider semantics. An app-level egui pointer-drag test verifies the action reaches the existing workspace width setter.
- At DB Pro source commit `948654524a6cd8a350a3eb972df33f2a5a1cd9ed`, query-output bottom/right splitters use rs-ui `Resizable` with inverted pointer coordinates to preserve existing drag direction and DB Pro size setters. Workspace tab selection nodes use rs-ui `Pressable` with `Tab`/`TabList` semantics; closed nodes are removed after the render pass.
- Sidebar wheel deltas are routed through rs-ui `ScrollState`; the retained offset drives egui content paint, while egui scrollbar interactions sync their resulting offset and layout extent back into the adapter.
- The shell and tabs continue to paint through egui. Tab and close-button activation are normalized through `Pressable`; native accessibility/focus behavior has not been verified.
- At DB Pro SHA `2041a1b1534a8cf6eb6eb776795db71a7ad24ae8`, connection/database/schema/Tables/table rows register stable-keyed `TreeItem` semantics with existing expanded/selected state; row activation is dispatched through rs-ui then returned to existing DB Pro interaction handlers. Explorer runtime ownership and its focused tests are isolated in `native_explorer_tree_runtime.rs`.
- At that same SHA, ArrowUp/ArrowDown map to rs-ui `MovePrevious`/`MoveNext`; only rows registered from the currently rendered viewport can participate. Native focus and accessibility remain unverified, and other schema-object folders/rows are not in the semantic tree.
- Historical finding at rs-ui SHA `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`: `VirtualGrid` accepts one fixed column extent, so horizontal virtualization for DB Pro's variable widths is not supported. The current pinned revision still has this fixed-width limitation; Stage 4.2 selection/resize integration is recorded below.

### Stage 4A — current result-grid ownership audit

Historical baseline source SHA: `5c1f42bad87c69f3cbfddcb921c557987706201e`; initial Stage 4 adapter implementation: `b224ec376ee50a93516d2c5691b700bcd9c63f8f`. Current DB Pro base SHA: `00a077e837fced6104bd34003340440847fe2f2a` plus the uncommitted working-tree changes for this continuation. Canonical rs-ui SHA: `f6e798d6cfa966b5344cf6a9de6c634563258eec`.

| Concern | Current owner / behavior |
|---|---|
| Total and projected row count | `UiQueryResult` owns all returned rows. `prepare_grid_cache` obtains filtered/sorted source-row indexes from `GridProjectionCache`; the displayed row count is `indexes.len()`, not `result.rows.len()` when a projection is active. |
| Visible row calculation | Before this continuation, `egui::ScrollArea::vertical().show_rows` chose the row range. The Stage 4 adapter now derives a range from the egui vertical scroll offset using rs-ui `ScrollState` and retained `VirtualGrid`; the egui scroll area still owns scroll input and offset. Only that range plus one-row overscan is sent to the row painter. |
| Column order / widths | DB Pro `TableDataState` owns persisted column order and widths. Widths default to 180 px and resize state clamps stored widths to 60–520 px. |
| Horizontal scrolling | egui `ScrollArea::horizontal` owns it. All ordered columns are still painted; horizontal virtualization is not implemented. |
| Selection and keyboard | DB Pro `TableDataState` owns selected cell/row/rows and range anchors. `SelectionModel` calculates row single/toggle/range operations from projection keys; DB Pro persists their result. `handle_grid_keyboard` and existing handlers retain navigation and clipboard ownership. |
| Resize | rs-ui `Resizable` calculates pointer delta and clamped value for result-column drag. DB Pro `TableDataState` remains authoritative for widths and persistence. |
| Typed-cell formatting | Rows keep `UiCell` references; the existing cell painter formats values as each visible cell is drawn. The adapter does not stringify or clone the dataset. |
| Paint path | `GridProjectionCache` → filtered/sorted indexes → virtual row range → `GridRows` → `draw_grid_row` / `draw_grid_cell` in egui. |

The benchmark covers fixed-width `VirtualGrid` window preparation at 10k, 100k and 1M rows with 50 columns. It measures `set_scroll_offset` plus `visible_cells()` allocation, not end-to-end frame preparation. Current timings and materialized-cell estimate are recorded in `VERIFICATION.md`; there is no compatible before/after baseline.

## Failure scenario / severity

- P2 integration risk: a future renderer cutover from eframe's current glow path needs separate startup, accessibility, and capture validation. This change does not perform that cutover.
- Historical P1 dependency/build blocker: rs-ui `db3cf2bfed3d29f0e1d19963462488ed9157f1ea` lacked APIs already called by Stages 1–3. It is resolved for this continuation by the exact Git pin `f6e798d6cfa966b5344cf6a9de6c634563258eec`.
- P2 tree-navigation limitation: rows omitted by egui viewport clipping are not registered for rs-ui focus traversal, so traversal across a large clipped tree is not established.
- P2 Stage 4 limitation: fixed-width `VirtualGrid` cannot safely virtualize DB Pro's variable-width columns. Horizontal virtualization remains blocked on a variable-width virtual-axis API.
- No proven defect in DB Pro business or database behavior was found in this integ

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

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
| `cargo fmt --all -- --check` at

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — rs-ui Runtime Integration

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex, implementation lane |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | DB Pro `977a5dd00e656e3529e18372edbc121496cb700e` |
| Source snapshot tree | `e317464c083188b75c04d5be1e1a9f4517086b38` (isolated index containing current Cargo.lock, crates/ui and crates/native-app; main index unchanged) |
| Canonical rs-ui SHA | `727d65d3957eb7bbfbd316e68a272ca123ab411c` |
| Branch / PR | main / no PR |
| Scope interpretation | Explorer tree viewport paint only, including rows, icons, selection/hover, indentation, chevrons, scroll viewport and feedback. Preserve DB Pro state/actions and the current host. |
| Out of scope | Sidebar chrome/search/context menus, topbar, tabs, editor, result grid, status bar, providers, CI and any next migration. |

## 2. Progress checkpoint

- Implemented: canonical font loading/named families; exact Git pin; rs-ui offscreen Explorer paint with cached Glow texture; canonical ScrollState offset and scrollbar input.
- Retained: existing row hitboxes, stable tree keys, DB Pro selection/actions, egui expansion ownership and its original persistent child namespace.
- Source gates and explicit GPU input/benchmark results are recorded in VERIFICATION.md. Native comparison covers three logical viewports, 1x/2x and normal/loading/error/empty.
- Remaining: visual sign-off for lighter text and revised feedback presentation; physical HiDPI monitor verification. The overall integration remains IMPLEMENTING and the Explorer slice is RUNTIME_VERIFY.
- Historical Welcome-only spike findings remain in FINDINGS.md and VERIFICATION.md; they describe earlier SHA f6e798d, not the current Explorer source.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact source | Snapshot tree `e317464c083188b75c04d5be1e1a9f4517086b38`; canonical rs-ui `727d65d3957eb7bbfbd316e68a272ca123ab411c` |
| Commit list | rs-ui `727d65d`: feat: support host fonts and named text families (owner-authorized push). DB Pro changes are uncommitted, preserving preceding Welcome WIP. |
| File / surface inventory | Explorer painter/scroll/feedback adapters; shared row geometry constants; native host initialization; deterministic capture fixtures; manifests/lock and existing active plan. |
| Acceptance mapping | Canonical text/renderer paint the tree viewport. GPU row input, wheel/thumb scrolling, persistent IDs and existing semantics/actions are checked. Visual superiority is not asserted. |
| Commands / artifacts | VERIFICATION.md; logs under /home/vietis/.agents/outputs/db-pro/testresults/explorer-renderer; images under /home/vietis/.agents/outputs/db-pro/artifacts/explorer-renderer |
| CI disposition | Not run; no CI configuration changed. |
| Known limitations | Changed frames incur synchronous GPU readback and host upload; 10k synthetic roots exceed a 16ms preparation budget; native primary GPU backend required; simulated rather than physical HiDPI; generic monospace can differ from host font. |
| Provider impact | PostgreSQL / SQLite: n/a, no service/domain/provider changes. Capture fixtures do not connect to a database. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed source | Tree `e317464c083188b75c04d5be1e1a9f4517086b38` and rs-ui `727d65d3957eb7bbfbd316e68a272ca123ab411c`; self-review only |
| Verdict | BLOCK for final visual acceptance; implementation available for review, no independent approval recorded |
| P0 / P1 / P2 | P0 0 / P1 0 / P2 3: text-weight acceptance, bridge/large-tree frame cost, release size above 50MiB target |
| Baseline vs regression | Existing large-tree traversal and binary-size debt remain. The bridge adds readback/upload cost and dependencies. It avoids full offscreen text materialization and skips unchanged framebuffer copies. |
| Resolved findings | EGL enumeration conflict; clipped error detail; changed persistent tree namespace. |
| Next task(s) | Review Explorer screenshots. Do not start text input, editor, grid paint or wider renderer migration. |

## 5. Research / audit handoff

- Date: 2026-10-02.
- Sources: DB Pro source tree and canonical rs-ui SHA above; installed egui_glow 0.29.1 shader; actual native framebuffer captures and benchmark logs.
- Facts: canonical coverage blending is linear into sRGB; egui_glow multiplies texture/color in gamma space. Matched-font captures have visibly different text weight.
- Inference: matching font files alone does not guarantee identical perceived text weight. Sharp placement is verified visually; sharper-than-egui remains unproven.
- Measurements exclude host Glow GPU upload/presentation in the headless GPU benchmark. Native window screenshots verify actual host composition.
- Physical Retina behavior and independent visual approval remain unverified.

## 6. Tổng kết (Vietnamese summary)

Đã triển khai riêng Explorer tree viewport bằng rs-ui, giữ state/actions và namespace expand/collapse hiện tại. API font đã push vào rs-ui và DB Pro pin exact SHA mới. Có tests, benchmark GPU và ảnh đối chiếu; chưa chốt visual acceptance vì chữ nhẹ nét hơn egui. Không migrate thêm surface hoặc cấu hình CI.

