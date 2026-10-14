# Docs reconcile — handoff (DocsReconcile)

Documentation-only reconciliation of the 2026-10-13 audit against `docs/plans`.
No code, no state changes, no `COMPLETED` promotions. Corrections appended in the
existing dated-correction style; history preserved.

## STATUS.md

- **New dated pointer block** near the top referencing `_audit-2026-10-13/` as the
  newest evidence set (102 captures / 34 surfaces × 3 sizes; `pg_integration`
  28/28 on PG 18.6; `mysql_integration` 8/8 on MySQL; SQL Server + SSH BLOCKED).
- **S6 ER Diagram row**: appended `[UPDATED 2026-10-13]` — the 2026-09-14
  "no position persistence" correction is superseded by
  `dbpro.native.er-layouts-v1` (`diagram/persistence.rs`, `app_storage.rs`,
  `app_lifecycle.rs`).
- **Native Visual Redesign row**: appended `[UPDATED 2026-10-13]` — "egui 0.36.2
  reverted" is stale; HEAD is on 0.36.2 (`65cbcf75` + repairs).
- **Data Grid Record View / Table Profile Quality / Table Indexes rows**: the
  two "baseline/inherited connection-form failures"
  (`new_postgresql_connection_defaults_to_tls_require`,
  `explicit_disable_selection_is_preserved_on_submit`) now PASS at HEAD — rows
  corrected with strike-through + dated note; fresh capture names referenced.

## FEATURE_AUDIT_2026-09-23.md

- §5 blank results grid: **[UPDATED 2026-10-13] FIXED** in `294ab089` (PR #308);
  recommendation #3 struck as DONE; `results_*.png` referenced.
- Release-rule parenthetical corrected: `ui-core-audit` P0s fixed in #307
  (verified, slice 05).

## Plan VERIFICATION / CHECKLIST files (13 plans)

Appended a dated `## Audit evidence update — 2026-10-13` section (verdict +
evidence pointers to `captures/` + live-provider runs) to:

- `er-diagram-canvas-ux/VERIFICATION.md` (SUPPORTS; persistence + `diagram_*.png`)
- `data-grid-record-view/VERIFICATION.md` (EVIDENCE_GAP; stale-failure correction;
  `table-record_*.png`)
- `table-profile-quality/VERIFICATION.md` (EVIDENCE_GAP; stale 992/2 correction;
  `table-profile_*.png`)
- `table-indexes-responsive-layout/VERIFICATION.md` (EVIDENCE_GAP; stale 992/2
  correction; `table-indexes/-foreign-keys/-constraints/-dependencies_*.png`)
- `ui-stitch-parity-pass/VERIFICATION.md` + `CHECKLIST.md` — T5 compare/safety-lock
  visual capture now DONE (`compare_*.png` at 3 sizes; SAFETY LOCK banner
  confirmed); checklist item ticked with original outage note struck but retained
- `settings-shell-visual-polish/VERIFICATION.md` (SUPPORTS; `settings_*.png`)
- `gallery-form-controls/VERIFICATION.md` — **empty stub replaced** with real
  state: per-scope evidence (input chrome precedence tests, frameless alert
  dismiss, cursor unit tests), screenshot inventory, fresh `gallery_*.png` /
  `connection-error_*.png` pointers, honest remaining gaps (manual focus/cursor
  pass, unrecorded gate logs)
- `query-editor-intelligence/VERIFICATION.md` (28/28 PG + 8/8 MySQL;
  `query/results/table-completion_*.png`; 29 ignored PG tests noted)
- `ui-query-workspace/VERIFICATION.md` (SUPPORTS; captures; stale 483-test note)
- `query-editor-zed-feel/VERIFICATION.md` (EVIDENCE_GAP; static captures noted,
  interactive gate still open)
- `welcome-query-editor-polish/VERIFICATION.md` + `CHECKLIST.md` (STALE-DOC;
  `welcome_top_inset_*` test no longer exists — annotated)
- `responsive-layout-primitives/VERIFICATION.md` (STALE-DOC; fully landed on main)
- `er-hardening-verification/CHECKLIST.md` — "Zoom clamped to [0.5, 2.0]"
  corrected (actual band `ER_MIN_ZOOM = 0.1`)
- `sidebar-header-and-delete-session/VERIFICATION.md` — nonexistent test
  `connection_switcher_opens_palette_scoped_to_connections` replaced with the
  real four: `events.rs:138`, `events.rs:179`, `app_tests.rs:3662`,
  `app_tests.rs:3674`
- `sqlite-view-column-introspection/VERIFICATION.md` — stale test name
  `introspect_view_columns_populated` → `introspection_populates_columns_for_sqlite_views`

## Verification performed

- `cargo test -p db-pro-ui --lib -- <four sidebar test names>` → **4 passed /
  0 failed** (fresh run, 2026-10-13).
- All test names and code citations verified against HEAD before writing
  (`events.rs:138/179`, `app_tests.rs:3662/3674`, `input/layout.rs:23`,
  `selection/ui.rs:572/578`, `tokens/component/input.rs:53`,
  `diagram/model.rs:19-20`).
- Docs-only change: no build/lint/fmt required.

## Left / notes

- `explorer-tree-perf` (F1: V/F/T folders still clone per frame — code gap, not
  doc), `sidebar-dbeaver-codex-layout` (plan describes unshipped 3-pane design —
  re-scope/close decision), `ui-data-grid-hardening` (missing STATUS.md row),
  `schema-triggers-runtime` S4 COMPLETE claim (needs code or retraction — owned
  by code-fix agents), `native-visual-redesign` Wave-17 stale text inside its
  own plan docs — flagged in slice reports, left for code-owning slices or owner
  decisions. These are code/re-scope items, not pure doc reconciliation.
- No plan states changed anywhere; all dated notes end with the current state
  reiterated.
