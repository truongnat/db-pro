# Audit — ER Diagram / Explorer / Sidebar slice (2026-10-13)

## 1. Slice scope

Plans covered: `er-diagram-canvas-ux`, `er-hardening-verification`,
`explorer-tree-perf`, `explorer-first-tab-redesign`,
`sidebar-dbeaver-codex-layout`, `sidebar-header-and-delete-session`,
`sidebar-content-full-width` (completed/). Code areas:
`crates/ui/src/diagram/`, `diagram_canvas_view.rs`, `diagram_view.rs`,
`diagram_state.rs`, `explorer_*.rs`, `sidebar_*.rs`,
`operation_events.rs`, `app_storage.rs`, `app_lifecycle.rs`.

## 2. Per-plan verdict table

| Plan | Claimed state | Supported state | Verdict | Key evidence |
|---|---|---|---|---|
| er-diagram-canvas-ux | IMPLEMENTING; all checklist items [x]; captures recorded | All 7 scope items exist in code; tests exist and run; committed screenshots | **SUPPORTS** (2 P2 nits) | `diagram_canvas_view.rs:101` (`allocate_painter`, no `ScrollArea`), `:390` (`zoom_delta`/`smooth_scroll_delta`), `:417-478` (node drag → `move_node`, spatial rebuild on drop), `:306-348` (auto-fit + snapshot restore), `:523-592` (minimap), `diagram/persistence.rs:5` (`dbpro.native.er-layouts-v1`), `app_storage.rs:52-61` (restore), `app_lifecycle.rs:73-74` (save), `diagram_canvas_view.rs:600` (`dragging_canvas_updates_pan`), `app_tests.rs:2073` (`diagram_move_node_updates_rect_and_incident_edge_bbox`); screenshots dir has 5 PNGs |
| er-hardening-verification | RUNTIME_VERIFY; 20 scope items verified; self-corrected to `PARTIAL` | Worker + spatial + viewport + LOD + scene pipeline all exist and are test-covered; runtime artifacts absent (self-acknowledged) | **EVIDENCE_GAP** | `diagram/layout.rs:37-172` (`ErLayoutWorker`, degraded mode `request_tx: Option`, `MAX_COALESCE_DRAIN`, `saturating_add` at `:114`), `diagram/spatial.rs` (`ErSpatialIndex`), `diagram/lod.rs`, `diagram/scene.rs`, `diagram/tests.rs` (100 `#[test]`s, 0 `#[ignore]`) |
| explorer-tree-perf | All impl items [x]; runtime verify unchecked | Cache/cull/default-closed/cap all exist; "defer V/F/T filters until open" NOT implemented — eager `filter_by_schema` clone runs per frame before folder check | **EVIDENCE_GAP** (one failed acceptance item) | `schema_explorer_state.rs:130-160` (`cached_tables` + `ExplorerNavCache`), `explorer_table_folder_view.rs:30-33` (default closed, auto-open on search), `explorer_schema_objects_view.rs:123-131` (clip cull + spacer), `app_types.rs:269` (`EXPLORER_MAX_TABLES=100`) + `:388` (cap); FAILED ITEM: `explorer_schema_object_folders_view.rs:42-53,91-102,141-…` collects `Vec` before `draw_category_folder` |
| explorer-first-tab-redesign | RUNTIME_VERIFY; merged `a9efb730` | Code changes present; runtime evidence has known gaps (self-acknowledged) | **EVIDENCE_GAP** | `explorer_surface_view.rs:56-64` (CONNECTIONS label), `explorer_connection_row_view.rs:50` (connecting row default-open), `:82` (`is_selected: is_connected`), `explorer_connection_node_view.rs:81` (`Connecting…` hint), `app_tests.rs:7095-7103` (wheel scroll test); screenshots: 5 PNGs (connecting/empty only, 1440/1920 height-capped) |
| sidebar-dbeaver-codex-layout | IMPLEMENTING; plan specifies 3 stacked `TopBottomPanel` sub-panes | Code implements a DIFFERENT design — unified hierarchical tree with connections as root nodes; the two persisted pane-height fields it added are dead state | **STALE-DOC** | `explorer_view.rs:11-16` ("unified hierarchical tree where connections are the root nodes"), `explorer_surface_view.rs:51-89` (single tree `ScrollArea`); `schema_explorer_state.rs:74-75` fields saved (`app_lifecycle.rs:128-132`) + restored (`app_storage.rs:132-138`) but never read by any draw code (grep across `crates/` = 0 draw-site hits) |
| sidebar-header-and-delete-session | IMPLEMENTING; 4 tests claimed | P1 delete-sibling fix present and tested; header merged-launcher matches FINDINGS decision; VERIFICATION cites one test that does not exist | **STALE-DOC** | `operation_events.rs:325-340` (`clear_deleted_connection` gates teardown on `deleted_was_active`), `events.rs:138,199` (2 delete tests), `app_tests.rs:3662` (`sidebar_header_launcher_opens_full_command_palette`), `:3674` (`deleting_sibling_connection_does_not_auto_reconnect_active`), `sidebar_chrome_view.rs:29-76` (merged launcher → `OpenCommandPalette`); MISSING: `connection_switcher_opens_palette_scoped_to_connections` (0 grep hits) |
| sidebar-content-full-width (completed/) | COMPLETED | Fix + 5 revert-verified guards + 3 committed PNGs all present | **SUPPORTS** | `app_tests.rs:7125` (`the_sidebar_paints_nothing_past_its_clip`), `:7186` (`the_driver_badge_is_fully_visible_in_the_navigator_tree`), `:7218` (`the_tree_row_spans_its_layout_width_not_its_clip`), `:7271` (`the_filter_field_border_is_painted_inside_the_field`); `explorer_surface_view.rs:68-82` (tree_width clip-bound); screenshots dir: 3 PNGs |

**ER persistence question — RESOLVED:** the STATUS.md S6 claim
("position persistence does **not** exist… no `dbpro.native.diagram*` keys",
corrected 2026-09-14) is now **superseded**. `er-diagram-canvas-ux` added real
per-connection persistence: `ErLayoutSnapshot` (positions/zoom/pan,
`diagram/persistence.rs:12-19`) under key `dbpro.native.er-layouts-v1`,
restored in `app_storage.rs:52-61`, written in `app_lifecycle.rs:73-74`,
applied via `ErGraph::apply_position_overrides` (`model.rs:188`,
`diagram_view.rs:131-134`), populated on node-drop (`diagram_canvas_view.rs:464-475`)
and on every pan/zoom (`sync_layout_snapshot`, `:496-507`). STATUS.md line 62
is STALE — it should be re-corrected to note persistence now exists.

## 3. UI findings

| # | Sev | Finding | Evidence |
|---|---|---|---|
| U1 | P2 | `cargo fmt --check` fails on committed `diagram_canvas_view.rs` (2 hunks at ~:33 and ~:54 — `matches!` wrapping). The plan's checked `cargo fmt --all -- --check` gate no longer holds at HEAD. | `cargo fmt --check -- crates/ui/src/diagram_canvas_view.rs` → non-empty diff output |
| U2 | P2 | Dead persisted state: `connections_pane_height`/`schemas_pane_height` are saved and restored every session but no draw code reads them (DBeaver 3-pane layout was never shipped; unified tree landed instead). | `schema_explorer_state.rs:74-75`, `app_lifecycle.rs:128-132`, `app_storage.rs:132-138`; zero draw-site grep hits |
| U3 | P2 | Dead parameter: `draw_views`/`draw_functions`/`draw_triggers` take `_count: usize` (call sites compute `count_by_schema`) but badge count is recomputed from the eagerly materialized Vec. Callers pay a count pass and the callee then pays a clone pass. | `explorer_schema_object_folders_view.rs:40,54,89,103,140`; call sites `explorer_schema_objects_view.rs:64-83` |
| U4 | n/a | `Color32::WHITE` in `diagram_view.rs:671,685,950` is used only for `layout_no_wrap` *measurement* galleys; painted text uses `theme.*` tokens. Not a violation. | `diagram_view.rs:961-966` (painted with `theme.text_primary`) |
| U5 | n/a | Status dots/icons in connection rows all resolve to theme tokens (`success`/`accent`/`danger`/`text_muted`). No hardcoded colors found in explorer/sidebar/diagram surfaces. | `explorer_connection_row_view.rs:118-138` |
| U6 | n/a | No `unwrap`/`expect` on data paths in scope — only inside `#[cfg(test)]` modules (`sidebar_history_view.rs:284`, `diagram/design_mode.rs:305`). | grep + `#[cfg(test)]` module bounds |

## 4. Feature findings

| # | Sev | Finding | Evidence |
|---|---|---|---|
| F1 | P2 | explorer-tree-perf acceptance NOT fully met: "Defer Views/Functions/Triggers materialisation until the folder is open" is unchecked behavior — `filter_by_schema` clones a `Vec<T>` every frame **before** the `draw_category_folder` open check, in all three folders. The P1 finding it was meant to fix (eager folder filters) is only partially fixed (Tables is deferred; V/F/T are not). | `explorer_schema_object_folders_view.rs:42-53` (views), `:91-102` (functions), `:141`+ (triggers); `schema_explorer_state.rs:114-119` (clone semantics) |
| F2 | P2 | `sidebar-dbeaver-codex-layout` plan describes a UI that does not exist (3 stacked resizable `TopBottomPanel` sub-panes, `draw_connections_pane`/`draw_schemas_pane`/`draw_objects_pane`). Shipped code is the unified tree. Either re-scope the plan to the unified tree or close it. | `explorer_view.rs:1-13`, `explorer_surface_view.rs:51-89`; no `draw_*_pane`/`TopBottomPanel` in `crates/ui/src` |
| F3 | P2 | `sidebar-header-and-delete-session` VERIFICATION.md lists 4 tests including `connection_switcher_opens_palette_scoped_to_connections` — that symbol does not exist anywhere in the repo (the merged-launcher design intentionally has no connections-scoped palette). The recorded "4 passed" run cannot be reproduced as written. | grep `connection_switcher_opens_palette` → 0 hits; existing tests: `events.rs:138,179`, `app_tests.rs:3662,3674` |
| F4 | P2 | er-hardening CHECKLIST asserts "Zoom clamped to [0.5, 2.0]" but `ER_MIN_ZOOM = 0.1` (canvas-ux widened the band). Checklist line is stale. | `diagram/model.rs:19-20`; checklist line 157 |
| F5 | n/a | er-hardening VERIFICATION runtime table records "Compact (<0.75×)" — superseded by the 0.45 threshold; the same file already self-corrects the headline to `PARTIAL`. Informational only. | `diagram/lod.rs:13` vs VERIFICATION:133 |

## 5. Captures taken

None. `DB_PRO_CAPTURE_TO=/tmp/audit-caps/er-explorer/er-diagram-1280x800.png
DB_PRO_WINDOW_SIZE=1280x800 DB_PRO_CAPTURE_DIAGRAM=1
./target/debug/db-pro-native` ran ~75s and produced no PNG: `mod capture` is
`#[cfg(feature = "capture")]` (`main.rs:21-22`) and the checked-in debug binary
(`target/debug/`, Oct 11) was built without it, so the env vars are inert —
exactly the failure mode documented in `er-diagram-canvas-ux/VERIFICATION.md`
item 3. Per the audit's no-rebuild rule, the capture driver **can** reach the
ER canvas (`capture.rs:71,427` `open_diagram_workspace_for_capture`) but only
from a `--features capture` build; none was available. Existing plan evidence
screenshots were inspected by file presence only (5 PNGs under
`er-diagram-canvas-ux/screenshots/`, 5 under `explorer-first-tab-redesign/`, 3
under `sidebar-content-full-width/`).

## 6. Missing evidence to reach COMPLETED

- er-diagram-canvas-ux: owner pointer-level drag smoke in the live app (drag → save → relaunch → same position); fix fmt drift; PG live-diagram capture.
- er-hardening-verification: real-artifact runtime numbers (pan/zoom smoothness, idle CPU <2%, memory stability across schema toggles) — the unit-test budgets are in place but the "live measured" table has no committed source artifact.
- explorer-tree-perf: runtime large-schema (≥500 tables) scroll verify on PG + SQLite; actually defer V/F/T materialization (or re-scope the item).
- explorer-first-tab-redesign: connected-tree and error-state captures; true 1440×900/1920×1080 viewports (host caps at 838 logical px); owner wheel-scroll check in a real build.
- sidebar-dbeaver-codex-layout: re-scope doc to unified tree or close; then runtime capture of the shipped tree.
- sidebar-header-and-delete-session: UI runtime evidence (header pill → palette; delete sibling while connected); fix the nonexistent test name in VERIFICATION.
- sidebar-content-full-width: manual runtime rows in CHECKLIST still unchecked (hover wash to drag line, `Failed:` hint full-width, resize clamp 220–380) — automated guards cover these but a live confirmation would close them.

## 7. P0/P1/P2 totals

- P0: 0
- P1: 0
- P2: 5 (U1 fmt drift, U2 dead persisted pane heights, U3 dead `_count` param, F1 V/F/T eager filters, F2/F3/F4 stale docs — counted as 3 distinct doc findings)

P2 total: **7** findings (4 code, 3 doc-accuracy). No P0/P1 in this slice.

## 8. Tổng kết bằng tiếng Việt

- Câu hỏi quan trọng đã được giải quyết: persistence vị trí node ER **đã tồn tại** trong code (`dbpro.native.er-layouts-v1`, per-connection, restore ở `app_storage.rs`, apply qua `apply_position_overrides`) — ghi chú S6 trong STATUS.md nói "không tồn tại" đã lỗi thời, cần sửa lại.
- `er-diagram-canvas-ux` được code hỗ trợ đầy đủ cả 7 mục scope (pan kéo, zoom con lăn về con trỏ, kéo node, auto-fit, minimap, toolbar compact, persistence); chỉ còn thiếu smoke-test kéo chuột thật và file `diagram_canvas_view.rs` hiện fail `cargo fmt --check` ở HEAD.
- `er-hardening-verification` kiến trúc (worker async, spatial index, LOD, scene) có thật và được ~114 test bao phủ, nhưng bằng chứng runtime "đo trực tiếp" không có artifact — tài liệu đã tự hạ xuống `PARTIAL`, đúng.
- `explorer-tree-perf` chưa đạt một tiêu chí nghiệm thu: Views/Functions/Triggers vẫn clone `Vec` mỗi frame trước khi kiểm tra folder mở — chỉ có Tables mới thực sự defer.
- `sidebar-dbeaver-codex-layout` là tài liệu lỗi thời: code đi theo unified tree (connection là root node), không phải 3-pane như plan; hai field `*_pane_height` persist nhưng không ai đọc → dead state.
- Không có capture mới: binary debug không build `--features capture` (module `capture` bị `#[cfg]` loại hẳn) nên env vars vô hiệu — đúng bẫy đã ghi trong VERIFICATION của canvas-ux; cần build lại với feature để chụp.
