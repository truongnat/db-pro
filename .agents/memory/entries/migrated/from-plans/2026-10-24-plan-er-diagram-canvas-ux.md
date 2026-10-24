---
schema_version: 1
id: "MEM-PLAN-er-diagram-canvas-ux-360205e6a586"
title: "Plan archive: Er Diagram Canvas Ux"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, er-diagram-canvas-ux]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "360205e6a586443069ac2fcb27029e6e8433e274b584c43c0a526f2b5b6fd9bb"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/er-diagram-canvas-ux/PLAN.md"
  - "docs/plans/active/er-diagram-canvas-ux/FINDINGS.md"
  - "docs/plans/active/er-diagram-canvas-ux/VERIFICATION.md"
  - "docs/plans/active/er-diagram-canvas-ux/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/er-diagram-canvas-ux/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# ER Diagram Canvas UX

State: IMPLEMENTING
Branch: `main` (owner override 2026-09-30 — no worktree)
Owner asks (2026-10-07): redesign the ER action row to a compact icon row;
the diagram is ugly, cannot drag, cannot navigate.

## Problem

Current canvas (`crates/ui/src/diagram_canvas_view.rs`) wraps the painter in
`egui::ScrollArea::both()` **and** applies a separate `pan` offset on top of the
viewport transform. The scroll area and the pan state fight each other: dragging
is routed to scrollbars, `pan` shifts content inside a scroll rect sized only to
`world_size * zoom`, and there is no wheel zoom, no node drag, no fit-on-open.

Visual issues: table titles truncated at a fixed 18 chars (`main.HumanResourc…`),
schema prefix wastes the title budget, edges share a single mid-x bend so
parallel edges and their labels stack on each other.

## Scope

1. **Infinite canvas** — remove `ScrollArea`; drag empty space pans; mouse wheel
   zooms toward the pointer; pinch/`zoom_delta` supported.
2. **Node drag** — grab a table card to move it; incident edge bboxes update
   live; spatial index rebuilt on drop.
3. **Auto fit** — fit-to-view on first graph ready, and debounced fit to the
   search-neighborhood subset after typing pauses.
4. **Minimap** — bottom-right overlay; click/drag navigates the viewport.
5. **Compact toolbar** — no `ER DIAGRAM` section label; left = context chips
   (tables · relationships · arranging), search + hop controls for large
   schemas; right = icon buttons (reset layout, design mode).
6. **Persisted layout** — manual node positions + zoom/pan per connection saved
   to `dbpro.native.er-layouts-v1` via `eframe::Storage`, restored on next open
   (falls back to auto-fit when absent).
7. **Visual polish** — two-line node header (name + `schema · n cols`) with
   pixel-measured truncation; edge lane offsets so parallel edges fan out;
   hovered/selected node keeps its edges accent, the rest dim.

## Out of scope

- Design Mode mutation semantics (existing panel kept, toggle restyled).
- PostgreSQL-specific diagram behavior (none — diagram is provider-agnostic;
  both providers feed the same `UiTableSummary`).
- Full force-directed layout (grid layout retained; manual positions override).

## Evidence plan

- Unit: node drag updates `world_rect` + incident edge bbox; wheel zoom anchors
  pointer; overrides survive worker rebuild; snapshot serde round-trip.
- `cargo build --release --locked -p db-pro-native`
- Captures via `capture` feature (`DB_PRO_CAPTURE_DIAGRAM` + seeded fixture) at
  1280×800, 1440×900, 1920×1080 dark + one light.


## Source: `FINDINGS.md`

# Findings — ER Diagram Canvas UX

## Root causes (pre-change SHA: see VERIFICATION)

1. **Dual navigation**: `draw_diagram_canvas` allocates a painter sized
   `world_size * zoom` inside `ScrollArea::both()`, then *also* applies
   `ctx.diagram.pan` in `ErViewport`. Scroll handles drag on the bar/gesture
   layer while pan shifts inside the allocated rect — pointer drags land on the
   scroll area, so the map feels locked ("không drag được") and content can be
   pushed outside the allocated rect ("không xem được").
2. **No node interaction**: positions come from `ErGraph::build` grid only;
   nothing hit-tests a node for drag. Click exists (`hit_test_node` →
   `OpenTable`) but no move.
3. **No wheel zoom / no fit-on-open**: zoom is only the floating −/+ widget.
4. **Truncation by chars**: header title hard-capped at 18 chars regardless of
   node width (280px ≈ 36+ chars at 13px), so every card reads
   `main.HumanResourc…`.
5. **Edge stacking**: single `bend_x = (from.x + to.x)/2` shared by every edge
   between adjacent columns; labels centered on the bend overlap each other.

## Owner direction (2026-10-07)

- Toolbar → compact icon row (context chips left, icon actions right).
- Interactions: node drag + pan/wheel zoom + auto fit + minimap.
- Manual positions persist across sessions.

## Findings during verification

- **Compact LOD band was too wide** (zoom < 0.75): at ~0.6, nodes rendered as
  large empty cards. Band lowered to < 0.45 so mid zooms render real columns.
- **Capture driver is feature-gated** (`--features capture`, off by default):
  `DB_PRO_CAPTURE_*` env vars are inert without it — the app opens normally and
  never writes a PNG, which masquerades as a "hang".
- **Two flaky first-paint stalls** in the capture path on macOS (fixed):
  invisible windows receive no `drawRect`, and both eframe window-state
  restore and the app's 3-frame startup `Maximized(true)` fought
  `pin_viewport`. See `VERIFICATION.md`.
- Persisted viewport/zoom is restored across runs — confirmed incidentally by
  the light capture rendering at a previously saved 60% zoom.


## Source: `VERIFICATION.md`

# Verification — ER Diagram Canvas UX

Status: evidence collected. All gates executed against the current working tree
(uncommitted on `main`, per owner override).

## Gates

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all -- --check` | PASS (exit 0) |
| check | `cargo check -p db-pro-ui` + workspace compile via clippy/test | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| test | `cargo test --workspace` | PASS — 1685 tests, 0 failed (41 ignored = live PG/MySQL/SQLServer/SSH fixtures needing external services) |
| native release | `cargo build --release --locked -p db-pro-native` | PASS (perf-scan built it: 32.5MB < 50MB) |
| perf audit | `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (4 executed: release build, binary size, check, clippy; ER/bench/db sections skipped — need window server/live DB) |
| clean code | `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 0 fail, 5 warnings (pre-existing ratchet size debt: `draw_diagram` 101 ln, `diagram_view.rs` 872 ln, `workspace_actions.rs` 1176 ln, `translate_cmd.rs`, `worker.rs` clones) |

## Runtime evidence

Capture binary must be built with `--features capture` (feature is off by
default; `DB_PRO_CAPTURE_*` env vars are inert without it — a plain
`cargo build` silently produces a binary that opens the app and never writes a
PNG). Screenshots land under `screenshots/`; the live "Sample E-Commerce"
SQLite connection (73 tables / 94 relationships) auto-connected and replaced
the seeded fixture, so the shots document the real schema.

| Viewport | Theme | File | Notes |
|---|---|---|---|
| 1280×800 | dark | `screenshots/er-1280x800-dark.png` | Auto-fit → 21% zoom, Compact pills, minimap, compact toolbar + zoom strip |
| 1280×800 | dark | `screenshots/er-1280x800-zoom1.png` | `DB_PRO_CAPTURE_DIAGRAM_ZOOM=1.3` → Detailed LOD: two-line headers, key/link icon markers, `FK → table.column` targets, crow's-foot + bar endpoints, edge label pills |
| 1440×900* | dark | `screenshots/er-1440x900.png` | Auto-fit → 22%; *macOS capped inner height at 838 logical px (display minus chrome) |
| 1920×1080* | dark | `screenshots/er-1920x1080.png` | Auto-fit → 22%; *height likewise capped at 838 |
| 1280×800 | light | `screenshots/er-1280x800-light.png` | Restored persisted viewport (60% = proof layout persistence survives restarts); Standard LOD renders full column rows in light theme |

Persistence is covered by `diagram_position_overrides_apply_by_table_key`,
`diagram_move_node_updates_rect_and_incident_edge_bbox`, and the storage
round-trip in `app_storage`; the 60% light capture is incidental runtime proof
that a viewport saved in an earlier run was restored. Manual drag/pan gestures
are exercised by `dragging_canvas_updates_pan` and the node-drag canvas tests;
pointer-level drag can only be fully confirmed by the owner in the live app.

## Bugs found & fixed during verification

1. `ErLod` banding: Compact covered zoom < 0.75, so at ~0.6 nodes painted as
   large empty cards (light capture exposed it). Threshold lowered to 0.45;
   tests updated (`lod_monotonic_transitions`, `zoom_changes_lod_not_graph`,
   `diagram_lod_transitions_and_rules`, scene test at zoom 0.5→0.3).
2. Capture-harness stalls (pre-existing, flaky): a `visible:false` window can
   receive no `drawRect` on macOS, so the first frame may never paint; eframe's
   persisted maximized state and `DbProApp`'s startup `Maximized(true)`
   (3 frames) both fought `pin_viewport`. Fixed by `with_visible(true)` +
   `persist_window:false` for capture runs and skipping the startup maximize
   when `DB_PRO_WINDOW_SIZE` is set.
3. Root cause of earlier "hang" runs: capture binary was rebuilt without
   `--features capture`; the env vars are inert and the app simply stays open.
   Documented above to prevent repeat.
4. Pan drag appeared dead (owner smoke test): `Response::drag_delta()` in egui
   0.29 is the per-frame pointer delta, not the distance since press, so
   `pan = origin + drag_delta()` snapped back every frame. Fixed by recording
   `(pan_at_grab, press_origin)` at `drag_started` and recomputing
   `pan = grab_pan + (pointer - press)`; the canvas drag test now asserts the
   full (74, 53) travel across two move frames — the old code would have left
   (10, 5) and failed.
5. "+N more columns" painted over the last real column row — it now takes the
   last row slot (renders `max_cols - 1` columns + the overflow line).
6. Owner review round 2: single click opened tables immediately; edges too
   faint and uninformative; cards thin on info. Now: double-click opens,
   single click pins the node's relationship highlight (click empty canvas to
   unpin), edges carry crow's-foot ("many" at FK side) + bar ("one" at PK
   side) endpoints, edge labels show `fk_col → pk_col` at Detailed always and
   at Standard while a node is highlighted, and column rows gain key/link
   icon markers plus `FK → table.column` targets at Detailed (and
   `PK·FK → target` for composite key columns); header subtitle gains
   row-count when introspection provides it.

## Provider accounting

- SQLite: live 73-table schema rendered + captured (dark + light, 3 viewports).
- PostgreSQL: diagram consumes the provider-neutral `UiSchemaSummary`; no
  provider-specific code path changed. PG live rendering not separately
  captured — no PG instance available on this host.

## Remaining

- Pointer-level drag persistence (drag → save → relaunch → same position) is
  covered by unit tests + snapshot restore evidence, but the physical
  mouse-drag gesture should be smoke-tested by the owner in the live app.
- Plan stays under `active/` pending owner review; move to `completed/` only
  after handoff acceptance.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/04-er-explorer-sidebar.md` verdict: **SUPPORTS** — all 7
scope items verified in code at HEAD (infinite canvas, wheel-zoom to cursor, node drag,
auto-fit, minimap, compact toolbar, per-connection persisted layouts). Persistence is
real: `ErLayoutSnapshot` under key `dbpro.native.er-layouts-v1`
(`diagram/persistence.rs`), restored in `app_storage.rs`, written in `app_lifecycle.rs`.

Fresh runtime evidence: `docs/plans/_audit-2026-10-13/captures/diagram_{1280x800,1440x900,1920x1080}.png` —
10 seeded tables, orthogonal FK edges, minimap, zoom readout (heights host-capped at
838 for the two larger widths). Live provider context: `pg_integration` 28/28 PASS on
PostgreSQL 18.6 and `mysql_integration` 8/8 PASS on MySQL this run (`CAPTURE-EVIDENCE.md`),
though the diagram still consumes the provider-neutral summary.

Still open: owner pointer-level drag smoke (drag → save → relaunch → same position) in
the live app, plus 2 P2 nits recorded in the slice report. State stays RUNTIME_VERIFY.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — ER Diagram Canvas UX

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Devin (SWE-2) · implementation lane |
| Issue(s) | owner request: redesign diagram action rows; diagram "ugly, can't drag, can't view" |
| Task state | `Review` |
| Baseline SHA | `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` — all assertions below are against the uncommitted working tree on top of this SHA (owner override: work directly on `main`, no commit requested yet) |
| Branch / PR | `main` · no PR |
| Scope interpretation | ER diagram surface in `crates/ui`: compact icon toolbar, infinite canvas (pan / wheel-zoom / node-drag), auto-fit, minimap, per-connection persisted layout, node/edge visual polish, LOD banding, capture-harness reliability for runtime evidence |
| Out of scope | other working-tree WIP (`welcome_surface_view`, data-grid record view, app menu, select/translate_cmd/query editor, table_data, select, welcome); archived React/Tauri frontend; live-DB capture for PostgreSQL |

## 2. Progress checkpoint

- Current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` (+ uncommitted diff)
- Completed acceptance rows:
  - [x] compact icon toolbar (context badges + search left, icon actions right)
  - [x] drag individual tables (spatial-index hit test + `ErGraph::move_node`)
  - [x] pan + wheel zoom (cursor-anchored; pinch via `zoom_delta`)
  - [x] automatic fit (initial fit or persisted viewport restore; refit on search settle)
  - [x] minimap (node rectangles + viewport outline; click/drag recenters)
  - [x] persisted table positions across sessions (`dbpro.native.er-layouts-v1`, per `connection_id`)
  - [x] visual polish: two-line headers with pixel-measured truncation, edge lane offsets, hover/drag dim+highlight, LOD banding fix
- Remaining acceptance rows: owner pointer-level smoke test of drag/pan in the live app (not automatable without a GUI driver)
- Findings / risks: see §4 — P2-4 added post-review: `drag_delta` is per-frame
  in egui 0.29; panning was broken until the press-anchored fix
- Tests already run: `cargo test --workspace` → 1685 passed / 0 failed / 41 ignored, exit 0 (ignored = live PG/MySQL/SQLServer/SSH fixtures)
- Dependency / blocker changes: none

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` + working-tree diff (uncommitted) |
| Commit list | none — no commits made; work sits in the working tree per owner override |
| File / surface inventory | `crates/ui/src/diagram/model.rs` — `move_node`, `recompute_world_bounds`, `apply_position_overrides`, `er_table_key`, `active_subset_bounds`; `crates/ui/src/diagram/persistence.rs` (new) — `ErLayoutSnapshot` serde type; `crates/ui/src/diagram/lod.rs` — thresholds Compact<0.45 / Standard<1.15 / Detailed; `crates/ui/src/diagram/viewport.rs` — world↔screen transforms; `crates/ui/src/diagram/mod.rs` — exports; `crates/ui/src/diagram/tests.rs` — LOD + model tests; `crates/ui/src/diagram_state.rs` — drag/hover/fit/search/saved-layouts fields; `crates/ui/src/diagram_canvas_view.rs` — rewritten: `allocate_painter` canvas, pan/zoom/node-drag input routing, overlay exclusion rects, minimap, search fit, snapshot sync; `crates/ui/src/diagram_view.rs` — compact toolbar, lane-offset edges + dimming + arrowheads, two-line node header + `truncate_to_width`, `fit_diagram_viewport`, `reset_diagram_layout`, world-anchored grid; `crates/ui/src/app_storage.rs` + `app_state.rs` — `restore_diagram_layouts` wiring; `crates/ui/src/app_lifecycle.rs` — layout save + startup-maximize skip under `DB_PRO_WINDOW_SIZE`; `crates/ui/src/app_tests.rs` — LOD test zoom value; `crates/ui/src/workspace_view.rs` — `connection_id` into `DiagramViewContext`; `crates/ui/src/workspace_actions.rs` — `open_diagram_workspace_for_capture` fixture; `crates/native-app/src/main.rs` — `persist_window:false` + `with_visible(true)` for pinned capture runs; `docs/plans/STATUS.md` — plan row; `docs/plans/active/er-diagram-canvas-ux/*` — plan docs + 5 screenshots |
| Acceptance mapping | drag → `handle_drag_input` + `ErGraph::move_node` + spatial rebuild on drop; pan/zoom → `handle_drag_input`/`handle_zoom_input` (cursor-anchored); auto-fit → `apply_initial_viewport`; minimap → `draw_minimap`; persistence → `ErLayoutSnapshot` + `saved_layouts` + storage key `dbpro.native.er-layouts-v1`; toolbar → `draw_diagram_toolbar` (badges + `compact_icon_button_active`/`enabled`) |
| Commands and counts | `cargo fmt --all -- --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo test --workspace` → 1685/0/41 exit 0; `cargo build --release --locked -p db-pro-native` exit 0 (32.5MB < 50MB budget); `bash .skills/perf-audit/scripts/perf-scan.sh` PASS (4 executed, ER/bench/db skipped — window server/live DB needed); `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` PASS 0 fail / 5 pre-existing ratchet warnings |
| CI run IDs / status | not run (no CI trigger on main in this session) |
| Known limitations | pointer-level drag gesture not exercised end-to-end (unit-tested + owner smoke pending); 1440×900 / 1920×1080 captures height-capped at 838 logical px by the host display; PG live rendering not captured (provider-neutral surface, no PG instance on host); `reset_diagram_layout` rebuilds the graph synchronously (same path as first load) |
| Migrations / config implications | new persisted key `dbpro.native.er-layouts-v1` in eframe storage (`ErLayoutSnapshot` per connection_id: zoom, pan, positions); new env knobs `DB_PRO_CAPTURE_DIAGRAM_ZOOM`, `DB_PRO_CAPTURE_DIAGRAM_LIGHT`; capture binary requires `--features capture` |
| Out-of-scope changes | `crates/native-app/src/capture.rs` macOS-menu hunk and all other unrelated WIP files were left untouched; temporary `eprintln!` instrumentation in `capture.rs`/`app_lifecycle.rs` fully reverted (verified: no `eprintln`/`dbg!` in diff) |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` + working-tree diff (self-review only — independent reviewer pending) |
| Verdict | `ACCEPT WITH P2` (self-review; not independent approval per AGENTS.md) |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 3 · inherited: 0 / 0 / 5 ratchet warnings |
| Findings | P2-1: `reset_diagram_layout` clears the graph and rebuilds synchronously — fine at 73 tables, could hitch at ~1000 (`diagram_view.rs`); P2-2: Compact LOD pills are minimal at very low zoom by design — grid overview only; P2-3: label galley `FontId::proportional(10.0)` is screen-space (intended) but labels only render when `Detailed` + `emphasized`, so dense graphs stay clean. Baseline P2s: 5 ratchet size/clone warnings (`draw_diagram` 101 ln, `diagram_view.rs` 872 ln, `workspace_actions.rs` 1176 ln, `translate_cmd.rs`, `worker.rs`) — pre-existing, unchanged by this diff |
| CI disposition | not run — no CI in session; all local gates green |
| Next task(s) unblocked | owner smoke test → plan move to `completed/`; other features' flaky capture runs are fixed by the harness changes (see VERIFICATION.md §"Bugs found") |

## 5. Research / audit handoff

n/a — implementation task.

## 6. Tổng kết

Đã làm lại toàn bộ canvas ER diagram: bỏ `ScrollArea` thay bằng infinite canvas (kéo canvas để pan, lăn chuột zoom theo con trỏ, kéo từng bảng), auto-fit khi mở, minimap điều hướng, toolbar thành icon row gọn, vị trí bảng lưu theo connection và khôi phục khi mở lại. Sửa bug LOD (Compact<0.45) làm card rỗng ở zoom giữa, và sửa flaky capture trên macOS (window `visible:false` không nhận redraw + maximize fight). Toàn bộ gate xanh: fmt/clippy/1685 test/release build/perf/clean-code; 5 screenshot thực tế (dark+light, 3 viewport, zoom 100% Detailed). Còn lại: owner cần smoke test kéo-thả bằng tay trong app thật và review độc lập; plan vẫn ở `active/` chưa chuyển `completed/`. Lưu ý quan trọng cho agent sau: binary capture **phải** build với `--features capture`, nếu không env `DB_PRO_CAPTURE_*` hoàn toàn vô tác dụng và app chỉ treo mở.

