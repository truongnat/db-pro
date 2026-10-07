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
