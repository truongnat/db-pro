# Native UI Stitch Parity Pass — Findings

Findings are evidence-backed. Severity: P0 catastrophic · P1 wrong/unsafe/broken
flow · P2 polish/maintainability.

## Pre-audit baseline (source inspection @ 52d3600c)

| # | Surface | Spec | Suspected delta | Sev | Task |
|---|---|---|---|---|---|
| F1 | Explorer object filter | 11 | name-contains only; no evaluation mode, no object-kind toggles, no persisted scope | P2 | T4 |
| F2 | Execution history | 12 | `query_history_entries` state exists; unified history surface (filters, day groups, record details) not confirmed in UI | P1* | T3 |
| F3 | Table workspace footer | 04, 16 | no Ready/Active Session/Auto-fallback/UTF-8/Ping telemetry strip | P2 | T1 |
| F4 | Queries sidebar | 14 | grouping/actions vs spec unaudited | P2 | T2 |
| F5 | Schema compare | 09 | diff summary + target-mismatch lock unaudited | P2 | T5 |

*P1 only if spec'd surface is wholly absent; downgrades to P2 if a basic
history view exists and only details are missing.

## T0 audit results

Captures: `/tmp/audit-table-dark.png`, `/tmp/audit-table-light.png`,
`/tmp/audit-diagram-dark.png` (1280×800). Source audit @ `52d3600c`.

| Surface | Evidence | Delta | Sev | Disposition |
|---|---|---|---|---|
| Table data (dark+light) | capture | Toolbar/tabs/pagination/type chips/TRUE·FALSE·NULL badges all present and close to spec; footer has conn+schema+ping+object counts but lacks spec's `Ready · Active Session · Auto-fallback · UTF-8 · Ping:` fields | P2 | T1 (footer fields only — toolbar already good) |
| ER diagram (dark) | capture | Grid canvas, PK/FK badges, edge labels, zoom controls, Design Mode, schema-map sidebar all present; no obvious spec 10 delta at this zoom | — | T6 optional, likely skip |
| Saved queries | source | Context menu already has Open-as-new/Insert/Copy/Rename/Delete (`sidebar_query_library_view.rs:172-198`); missing per-item "Source: conn · db · schema" line + Recent SQL + snippet shortcut codes from spec 14 | P2 | T2 |
| Execution history | source | `query_history_entries` exists, persisted via `app_lifecycle.rs:61`, surfaced only as palette rows + a count in `query_output_actions_view.rs:149` — **no unified history surface** (spec 12: filters, day groups, record details, open-as-query) | P1 | T3 — primary feature of this pass |
| Schema compare | source | Diff groups + Apply button + data-row states exist; spec 09 summary counts (`1 Added · 1 Changed · 0 Removed`) and target-mismatch safety-lock banner not found | P2 | T5 |
| Files workspace | source | Trusted/Untrusted badge + toggle exists (`files_surface_view.rs:181-195`) | — | skip |
| Explorer filter | source | name-contains only (`explorer_schema_object_folders_view.rs:41`); spec 11 workbench (mode/kinds/persisted) absent | P2 | T4 |

## Post-implementation notes

- **T5 target-lock semantics**: the lock compares the `connection_name · active_schema`
  string recorded at plan time against the live session at apply time. This is the
  honest identity available in the UI model — connection *ids* are not threaded
  through the compare view. Re-diffing clears the recorded target so a stale lock
  cannot outlive its plan.
- **T5 capture gap**: `DB_PRO_CAPTURE_COMPARE` helper is committed, but during the
  session the host's windowing stopped emitting `ViewportCommand::Screenshot`
  replies — even the previously-working `DB_PRO_CAPTURE_HISTORY` env produced no
  PNG (process stays alive, no `capture: wrote framebuffer` in the log). The lock
  is covered by unit tests; visual evidence remains pending.
- **T2 scope decision**: saved-query "Source:" line skipped — no source binding in
  `UiSavedQuerySummary` (id/name/sql/folder only).
- **Concurrent-session hazard observed**: a file watcher or other session rewrote
  `result_grid_cell_editor_surface_view.rs` mid-task during earlier phases; final
  diffs were always re-verified before commit.
