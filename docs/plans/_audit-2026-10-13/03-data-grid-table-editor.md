# Audit — Data Grid / Table Editor slice (2026-10-13)

## 1. Slice scope

Plans covered (6 active + 1 completed):

- `table-data-editor-hardening` (active, RUNTIME_VERIFY)
- `data-grid-record-view` (active, RUNTIME_VERIFY)
- `table-profile-quality` (active, RUNTIME_VERIFY)
- `table-indexes-responsive-layout` (active, RUNTIME_VERIFY)
- `ui-data-grid-hardening` (active, no state field; absent from STATUS.md)
- `responsive-layout-primitives` (active, IMPLEMENTING)
- `table-data-mutation-index-mapping` (completed/)

Code areas verified: `crates/ui/src/change_set.rs`, `table_mutation_state.rs`,
`table_mutation_actions.rs`, `table_mutation_dialogs_view.rs`,
`table_conflict_dialog_surface.rs`, `table_mutation_dialog_surface.rs`,
`table_events.rs`, `table_data_state.rs`, `table_data_surface_view.rs`,
`result_grid_view.rs`, `result_grid_edit.rs`,
`result_grid_record_surface_view.rs`, `result_grid_record_tests.rs`,
`result_grid_cell_surface_view.rs`, `components/table/ui.rs`,
`components/responsive_layout/*`, `component_gallery_{view,inputs,surfaces,feedback}.rs`,
`table_profile_surface_view.rs`, `table_indexes_surface_view.rs`,
`result_grid_toolbar_view.rs`, `table_data_mutation_toolbar_view.rs`,
`table_data_pagination_view.rs`, `components/legacy.rs`,
`crates/core/src/application/table_mutation_execution.rs`, `table_data_service.rs`.

Baseline-failure question (from record-view FINDINGS §9 / STATUS.md rows 75–77):
the two cited failures are `new_postgresql_connection_defaults_to_tls_require`
and `explicit_disable_selection_is_preserved_on_submit`
(`crates/ui/src/app_tests.rs:372`, `:467`). **Both PASS at HEAD today**:

```
cargo test -p db-pro-ui --lib new_postgresql_connection_defaults_to_tls_require → ok (1 passed)
cargo test -p db-pro-ui --lib explicit_disable_selection_is_preserved_on_submit → ok (1 passed)
```

Every plan doc that still carries "992 pass / 2 connection-form failures" or
"1656 pass / 2 inherited fail" is stale on that specific fact.

## 2. Per-plan verdict table

| Plan | Claimed state | Supported state | Verdict | Key evidence |
|---|---|---|---|---|
| table-data-editor-hardening | RUNTIME_VERIFY; RowIdentity/ChangeSet, 3-way conflict, composite-PK reload, atomic rollback wired UI→runtime; PG+SQLite verified | All pipeline pieces exist and focused tests pass at HEAD; SQLite live evidence logged; **live PG mutation evidence + native conflict/reload captures still missing** | EVIDENCE_GAP | `change_set.rs:4-304`; `table_conflict_dialog_surface.rs:54-165` (Original/Local/DB Current + Keep Mine/Use Database/Retry/Discard); `table_events.rs:194-236` (single-row merge); `table_mutation_actions.rs:348-360` (conflict→dialog+targeted reload); `table_mutation_state.rs:105-128` (composite-PK equality filters); tests `test_composite_pk_targeted_reload_and_merge`, `test_conflict_keep_mine_and_use_database_resolution_actions`, `conflict_failure_has_distinct_code_and_user_action_message`, `apply_is_blocked_while_a_validation_error_exists` all PASS today; `core table_mutation_execution.rs:68-71` statement-index remap |
| data-grid-record-view | RUNTIME_VERIFY; rail + single-record + click-to-copy; 6 focused tests; 15 captures | Fully wired: 40px rail left of `#`, Record bottom-anchored, exactly-one-row gate, staged effective values, click-to-copy via `OutputCommand::CopyText`; 7 record tests pass; 3-width captures exist in plan evidence/ | EVIDENCE_GAP | `table_data_surface_view.rs:5,37-77,84-111`; `result_grid_view.rs:73-82` (record replaces grid; exits when selection filtered out); `result_grid_edit.rs:295-330` (staged row build + copy dispatch); `result_grid_record_surface_view.rs:14-64` (Field\|Value shared Table, click-to-copy); `evidence/{grid,record,loading,error,empty,changed}-*.png`; `cargo test -p db-pro-ui --lib record` → 7 record tests pass |
| table-profile-quality | RUNTIME_VERIFY; content-width measurement preserves Range/Avg/Sum; badge borders; 992/2 suite | Implemented: BigDecimal extrema, consistent date formatting, sample-scoped labels, dedicated empty states, measured content widths incl. full numeric text, horizontal scroll, theme-only colors; 3 profile tests pass; "2 connection-form failures" claim now stale (both pass) | EVIDENCE_GAP | `table_profile_surface_view.rs:142-164` (BigDecimal ordering), `:179-246` (`profile_text_width` + `TableColumn::content_width`), `:253-258` (h-scroll), `:381-399` (pattern labels); `cargo test -p db-pro-ui --lib profile` → `long_numeric_summary_is_rendered_without_ellipsis_or_cell_clipping` + 2 others PASS |
| table-indexes-responsive-layout | RUNTIME_VERIFY; 6 columns fit 928pt viewport; hover disclosure; shared headers | Implemented: viewport-derived fractional widths (min 640pt table), `.truncate()` + `on_hover_text`, singular/plural count, modal removed (passive label asserted); geometry regression test passes at HEAD | EVIDENCE_GAP | `table_indexes_surface_view.rs:154-164` (`index_columns` 0.20/0.21/0.10/0.16/0.15/0.18 of `max(avail-2, 640)`), `:141-152` (passive `index_name_label`), `:223-235` test `index_columns_fit_the_standard_table_detail_viewport` PASS today; captures `indexes-*-{1280,1440,1920}*.png` in evidence/ |
| ui-data-grid-hardening | Canonical Buttons for all grid/table toolbars + pagination; non-regression tests | Supported: zero `compact_button_with_icon`/`compact_icon_button`/`ui.button` callers in grid surfaces; canonical `Button` + `ButtonSize`/`ButtonVariant` throughout; source-scan non-regression test present; BUT legacy compact fns are now dead code (see §4) | SUPPORTS (code); STALE-DOC (evidence + STATUS row missing) | `result_grid_toolbar_view.rs:58-188`, `table_data_mutation_toolbar_view.rs:42-239`, `table_data_pagination_view.rs:38-99` all `Button::new`; `components/mod.rs:128-153` `primary_native_surfaces_do_not_reintroduce_raw_buttons`; VERIFICATION claims "483 tests" vs measured 1,126 db-pro-ui lib tests today |
| responsive-layout-primitives | IMPLEMENTING on `feature/responsive-layout-primitives`; Container/Row/Col/ResponsiveGrid planned; checklist items unchecked | **Fully landed on `main`** (no feature branch): `Container`, `ResponsiveGrid`, `container_width`, `grid_metrics` implemented + exported; Gallery connection form + outer container migrated; 7 unit tests pass; no separate Row/Col types (Grid covers it per approved direction) | STALE-DOC | `components/responsive_layout/ui.rs:69-195`; `components/mod.rs:88` export; `component_gallery_view.rs:257` `Container::new().gutter(SPACE_LG).max_width(1120.0)`; `component_gallery_inputs.rs:112` `ResponsiveGrid::new(280.0).gap(SPACE_LG).max_columns(2)` (+4 more gallery consumers); `cargo test -p db-pro-ui --lib responsive_layout` → 7/7 PASS; CHECKLIST items `[ ]` for code that exists |
| table-data-mutation-index-mapping | COMPLETED; statement_index remapped only in Statement phase; validation/begin keep 0 | Verified at HEAD: remap gated on `phase == Statement`, `validation_failure` emits `statement_index: 0`; all 4 regression tests pass today | SUPPORTS | `table_mutation_execution.rs:67-72`, `:150-157`; `cargo test -p db-pro-core --lib apply_mutations_detailed` → 4/4 PASS |

## 3. UI findings

- **P2** `crates/ui/src/components/legacy.rs:406-473` — `compact_button_with_icon`,
  `compact_button_with_icon_enabled`, `compact_icon_button`,
  `compact_icon_button_enabled` have **zero callers** (only the `lib.rs:39`
  re-export). The very hardening the plan shipped left its target helpers as
  dead code; contract says obsolete code should be removed.
- **P2** `crates/ui/src/change_set.rs:88-91` — `set_target_table` has zero
  callers and is explicitly kept via `#[allow(dead_code)]` ("no active caller").
  Dead code on a data path.
- **P2** `crates/ui/src/change_set.rs:210` — `remove_insert` carries
  `#[allow(dead_code)]` but **is** called
  (`table_mutation_dialogs_view.rs:60`, `app_tests.rs:6267`). Stale/misleading
  attribute + comment claiming "no row target".
- **P2** `result_grid_view.rs:81` — when the selected row is filtered out,
  `record_view_open` is silently reset; intended per plan (falls back to Grid),
  but no user feedback is emitted. Acceptable; noted for completeness.
- No hardcoded colors found in audited grid/table surfaces: cell fills all
  derive from `theme.danger/warning/accent*` (`result_grid_cell_surface_view.rs:26-44`),
  conflict dialog uses tokens (`table_conflict_dialog_surface.rs`), only
  `Color32::TRANSPARENT` appears (`table_data_toolbar_surface_view.rs:530`).
- Cell clip/badge borders: body cells clip to
  `inner_rect.expand(1.0).intersect(cell_rect)`
  (`components/table/ui.rs:420-423`) — stroked badges paint into padding
  without neighbor bleed, matching the profile-quality claim. Verified in source.
- No `unwrap()`/`expect()`/`panic!` on data paths in audited files (only in
  `#[cfg(test)]` code).

## 4. Feature findings

- **P1** table-data-editor-hardening: live **PostgreSQL** mutation evidence is
  still unevidenced (STATUS.md row 54 correction stands). PG integration tests
  remain `#[ignore]`d without a fixture — per AGENTS.md they do not count.
- **P2** `table_events.rs:221-228`: targeted reload merge locates the row via
  linear `.position(|row| row_matches_identity(...))` over the current page,
  **not** the "PK column-index map / direct match" the checklist describes.
  Behaviorally equivalent (row-matched, no result clone); doc slightly
  overstates the data structure.
- **P2** ui-data-grid-hardening has **no STATUS.md row** and no State field in
  PLAN.md; VERIFICATION totals (483 tests) are far below current measured
  suite (1,126 db-pro-ui lib tests, `cargo test -p db-pro-ui --lib -- --list`).
- **P2** responsive-layout-primitives: plan claims branch
  `feature/responsive-layout-primitives` and `IMPLEMENTING` with unchecked
  primitives; code is fully on `main` and exported. Checklist, PLAN task list,
  and STATUS state are stale (STALE-DOC). No Row/Col types exist —
  `ResponsiveGrid` covers the documented "equivalent responsive Grid API".
- **P2** record-view: `egui-winit` clipboard bridge ignores empty text —
  click-to-copy of an empty value is a documented no-op
  (`result_grid_record_surface_view.rs:30` renders "(empty)" display but the
  copy path sends the raw string; see `record_click_copies_null_and_reports_empty_value`
  test expectations). Known limitation, correctly recorded in FINDINGS.
- **P2** `on_table_row_reloaded` handles "row deleted" by message only
  (`table_events.rs:204-210`); staged state for the deleted row is retained but
  the user gets no conflict affordance — retry flags are just cleared.
- Test coverage confirmed live (not `#[ignore]`d): record-view 7 tests,
  conflict 3 tests, composite-PK reload 1, profile 3, indexes geometry 1,
  responsive_layout 7, core index-mapping 4 — **all pass at HEAD**.

## 5. Captures taken

**None.** `DB_PRO_CAPTURE_*` envs are gated behind `#[cfg(feature = "capture")]`
(`crates/native-app/src/main.rs:21-22`). Both `target/debug/db-pro-native` and
`target/release/db-pro-native` at HEAD were built **without** the capture
feature: envs were silently ignored and no PNG was written (60s/75s runs, exit
via timeout). A `cargo build -p db-pro-native --features capture` build is a
workspace compile this audit is not allowed to run. Existing fixture captures
under `docs/plans/active/data-grid-record-view/evidence/` (grid/record/
loading/error/empty/changed at 3 widths), `table-profile-quality/evidence/`
(profile-width/dynamic-numeric, structure-header at 3 widths), and
`table-indexes-responsive-layout/evidence/` (indexes/foreign-keys/constraints/
dependencies headers at 3 widths) were inventoried instead — all heights capped
at 838 logical px per recorded host limit.

## 6. Missing evidence to reach COMPLETED (per plan)

- table-data-editor-hardening: live PG parameterized mutation run (insert/
  update/delete, composite PK, 0-row conflict, rollback) + native captures of
  conflict dialog, targeted reload, pending-changes dialog at required sizes.
- data-grid-record-view: exact-height 1440×900/1920×1080 captures (host cap
  workaround), live PG/SQLite loaded-result record flow, physical OS clipboard
  + accessibility verification.
- table-profile-quality: loading/error/empty-state capture matrix at
  1440×900/1920×1080, hover-disclosure + horizontal-scroll interaction
  evidence, VoiceOver pass.
- table-indexes-responsive-layout: empty/loading-state captures, interactive
  filter/category/search runtime matrix, detail-dialog visual check.
- ui-data-grid-hardening: refresh VERIFICATION to current workspace totals;
  remove dead `compact_*` helpers; add STATUS.md row.
- responsive-layout-primitives: independent review sign-off; sidebar open/closed
  captures at 3 widths around column transitions; then flip stale checklist items.
- table-data-mutation-index-mapping: none — completion claim verified.

## 7. P0/P1/P2 totals

- P0: 0
- P1: 1 (missing live-PostgreSQL mutation evidence for the staged pipeline)
- P2: 7 (legacy dead buttons; `set_target_table` dead code; stale
  `#[allow(dead_code)]` on `remove_insert`; doc vs linear-scan merge;
  ui-data-grid-hardening missing STATUS row + stale totals; responsive-layout
  docs badly stale; empty-clipboard no-op limitation)

## 8. Tổng kết bằng tiếng Việt

- Pipeline staged-mutation (RowIdentity/ChangeSet, conflict 3 chiều, reload
  composite PK, rollback atomic) **đã có đủ trong code và đang nối UI→runtime**;
  mọi test trọng điểm chạy PASS tại HEAD — thiếu duy nhất bằng chứng PG live.
- Record view đúng mô tả: rail 40px trái cột `#`, Record neo đáy, single-row
  gate, copy label/value dùng staged value; test record 7/7 PASS, capture
  fixture 3 độ rộng có sẵn trong `evidence/`.
- Hai test connection-form được STATUS/VERIFICATION ghi "baseline failing" —
  `new_postgresql_connection_defaults_to_tls_require` và
  `explicit_disable_selection_is_preserved_on_submit` — **hiện PASS cả hai**;
  các con số "992 pass / 2 fail" và "1656 pass / 2 inherited fail" là STALE.
- `ui-data-grid-hardening` đã hoàn thành về code (toàn bộ toolbar/pagination
  dùng canonical `Button`), nhưng để lại 4 hàm `compact_*` trong `legacy.rs`
  không caller — dead code, và VERIFICATION/STATUS lỗi thời.
- `responsive-layout-primitives` **đã integrate đầy đủ trên main** (Container +
  ResponsiveGrid, 7 test PASS, gallery dùng ở 6 chỗ); checklist/PLAN/branch
  trong docs là STALE-DOC chứ không phải half-landed.
- Không lấy được capture mới: binary debug lẫn release đều thiếu feature
  `capture`; ghi nhận là blocker cho evidence mới, không phải lỗi sản phẩm.
