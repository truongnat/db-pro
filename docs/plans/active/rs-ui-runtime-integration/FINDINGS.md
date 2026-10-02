# Findings — rs-ui Runtime Integration

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
