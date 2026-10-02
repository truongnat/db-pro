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
- At rs-ui SHA `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`, `VirtualGrid` only accepts one fixed column extent, so it cannot calculate the viewport for DB Pro's variable column widths. The Stage 4 working-tree adapter routes vertical range calculation through `VirtualGrid`/`ScrollState`; egui remains the host scroll input and painter. Horizontal virtualization and rs-ui selection/resize integration remain deferred.

### Stage 4A — current result-grid ownership audit

Baseline source SHA: `5c1f42bad87c69f3cbfddcb921c557987706201e`; the adapter continuation is uncommitted, so these implementation notes describe the working tree and have no resulting commit SHA.

| Concern | Current owner / behavior |
|---|---|
| Total and projected row count | `UiQueryResult` owns all returned rows. `prepare_grid_cache` obtains filtered/sorted source-row indexes from `GridProjectionCache`; the displayed row count is `indexes.len()`, not `result.rows.len()` when a projection is active. |
| Visible row calculation | Before this continuation, `egui::ScrollArea::vertical().show_rows` chose the row range. The Stage 4 adapter now derives a range from the egui vertical scroll offset using rs-ui `ScrollState` and retained `VirtualGrid`; the egui scroll area still owns scroll input and offset. Only that range plus one-row overscan is sent to the row painter. |
| Column order / widths | DB Pro `TableDataState` owns persisted column order and widths. Widths default to 180 px and resize state clamps stored widths to 60–520 px. |
| Horizontal scrolling | egui `ScrollArea::horizontal` owns it. All ordered columns are still painted; horizontal virtualization is not implemented. |
| Selection and keyboard | DB Pro `TableDataState` owns selected cell/row/rows and range anchors. `handle_grid_keyboard` and existing grid interaction handlers own navigation, range behavior and clipboard actions. No rs-ui `SelectionModel` is available in the current checkout. |
| Resize | DB Pro's existing width state and grid header interactions remain authoritative. No rs-ui `Resizable` grid API is available in the current checkout. |
| Typed-cell formatting | Rows keep `UiCell` references; the existing cell painter formats values as each visible cell is drawn. The adapter does not stringify or clone the dataset. |
| Paint path | `GridProjectionCache` → filtered/sorted indexes → virtual row range → `GridRows` → `draw_grid_row` / `draw_grid_cell` in egui. |

The new benchmark cases cover retained `VirtualGrid` visible-range preparation at 10k, 100k and 1M rows with 50 fixed-width columns. They have not compiled or produced timing/allocation results; there is no before/after performance claim. The current benchmark does not establish variable-width horizontal virtualization or measure end-to-end frame preparation.

## Failure scenario / severity

- P2 integration risk: a future renderer cutover from eframe's current glow path needs separate startup, accessibility, and capture validation. This change does not perform that cutover.
- P1 dependency/build blocker: the available immutable rs-ui revision lacks APIs already called by Stages 1–3; the focused DB Pro test fails to compile when resolving against that revision. `/data/dev/projects/ui-runtime-foundation` is absent, so the compatible source used for the earlier Stage 1–3 verification is unavailable here.
- P2 tree-navigation limitation: rows omitted by egui viewport clipping are not registered for rs-ui focus traversal, so traversal across a large clipped tree is not established.
- P1 Stage 4 verification blocker: the adapter and benchmark cannot be compiled or measured until DB Pro can resolve a compatible rs-ui revision containing APIs already consumed by Stages 1–3.
- P2 Stage 4 limitation: fixed-width `VirtualGrid` cannot safely virtualize variable-width columns; the available runtime also has no selection or resize models.
- No proven defect in DB Pro business or database behavior was found in this integration audit.

## Decision

Use rs-ui core/runtime as the shell layout authority through a thin adapter, retaining egui for host input and painting. Do not depend on `ui-renderer` or `ui-window` in the first slice.

For Stage 4, keep the current local dependency while the required rs-ui API is unavailable. Do not pin `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`: it lacks APIs used by Stages 1–3. Pin after a compatible rs-ui commit is published and confirms variable-width grid support if horizontal virtualization is in scope.

Keep Stage 4 incremental: DB Pro remains the source of truth for result projection, typed cells, order, widths, selection and clipboard; egui remains host/painter. Complete resize, selection, horizontal virtualization and measured correctness/performance only after a compatible rs-ui API is available.

## Provider impact and runtime testability

PostgreSQL: N/A; SQLite: N/A. This change does not touch database behavior. Shell screenshots from the earlier layout pass exist at 1280×800 and a constrained 1440×838 logical capture; 1920×1080 was not captured. Native product smoke is still pending. The adapter unit and app-level drag tests are not OS-level input proof.
