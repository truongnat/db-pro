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
- No proven defect in DB Pro business or database behavior was found in this integration audit.

## Decision

Use rs-ui core/runtime as the shell layout authority through a thin adapter, retaining egui for host input and painting. Do not depend on `ui-renderer` or `ui-window` in the first slice.

For Stage 4, pin rs-ui core/runtime to exact Git revision `f6e798d6cfa966b5344cf6a9de6c634563258eec`; Cargo.lock resolves that same source. The older `db3cf2bf...` incompatibility finding applies only to that historical revision.

Keep Stage 4 incremental: DB Pro remains the source of truth for result projection, typed cells, order, widths, selection and clipboard; egui remains host/painter. Vertical windowing, selection behavior, column resizing and benchmark execution are implemented; native runtime verification and variable-width horizontal virtualization remain open.

## Provider impact and runtime testability

PostgreSQL: N/A; SQLite: N/A. This change does not touch database behavior. Shell screenshots from the earlier layout pass exist at 1280×800 and a constrained 1440×838 logical capture; 1920×1080 was not captured. Native product smoke is still pending. The adapter unit and app-level drag tests are not OS-level input proof.

## Current continuation — rs-ui pin and Stage 4.2

Source: DB Pro base commit `00a077e837fced6104bd34003340440847fe2f2a` plus the uncommitted working-tree changes; rs-ui exact revision `f6e798d6cfa966b5344cf6a9de6c634563258eec`.

- `draw_body` passes `indexes.len()` from `GridProjectionCache` to the retained adapter; virtual total rows therefore match the filtered/sorted projection rather than raw `UiQueryResult.rows`.
- The adapter returns a clamped visible range with one-row overscan. Tests cover zero/single row, middle/end offsets, huge offsets, and 10k/100k/1M arithmetic; returned ranges remain within the projection.
- `GridRows` borrows projection indexes/result/order/widths. The painter reads and formats only visible typed `UiCell` values; it does not clone the projection or stringify the dataset.
- `ResultGridVirtualRuntime` is retained in `TableDataState` across frames. It replaces `VirtualGrid` only when row count changes; viewport and offset are refreshed on each call.
- egui `ScrollArea` remains the vertical input/offset owner. The adapter derives the visible window from that offset and its spacer cancels the egui content transform for painting; it does not add an independent second scroll offset.
- `SelectionModel` operates on projected source-row keys for single, toggle and range operations. Selected cells/rows/anchors remain stored in DB Pro `TableDataState`; clipboard and keyboard commands remain on the existing path.
- Column drag passes absolute pointer positions into rs-ui `Resizable`; the returned clamped value is written into DB Pro's persisted width vector. The rs-ui node stores only transient drag behavior state.
- Horizontal virtualization remains blocked on variable-width virtual-axis support. Runtime UI evidence is still needed before Stage 4 can advance beyond IMPLEMENTING.
