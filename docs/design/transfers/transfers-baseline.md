# Transfers — source baseline

- Source date: 2026-09-24
- Baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: native Rust/egui Transfers activity, transfer core/adapters, query-result export, backup/restore paths, and product-doc drift. Source line references below are at the stated SHA.
- No source code, tests, plans, or runtime state were changed or exercised for this research.

## Finding

Transfers is already a first-class activity in the native rail, and the repository contains a generic row-batch engine plus file adapters. The activity currently exposes a developer/demo harness, not a user workflow for moving a real table/query/file into or out of a live PostgreSQL or SQLite database. Backup/restore is functional on a separate Settings surface. The current implementation is therefore more than a blank placeholder, but materially short of the Phase C user outcome.

**Recommendation:** retain the Transfer rail. Replace the synthetic harness as the primary user surface with Import, Export, Backup, Restore, and Jobs entry points. Keep dev/test harnesses out of the normal user workflow. Treat Phase F compare/migration as related destinations, not as extra Phase C transfer scope.

## Surface and actual call path

- `crates/ui/src/activity_bar_view.rs:85-96` places `Activity::Transfers` in the native activity rail; `crates/ui/src/palette_actions.rs:19-20` also routes the palette action there.
- `crates/ui/src/transfer_activity_surface_view.rs:27-36,50-65,68-94,162-175,232-297` draws a backup-settings card, synthetic table seed, masking sample, synthetic harness controls, and an in-memory job list. The empty-state copy explicitly offers the harness and says import/export formats will land later.
- `crates/ui/src/transfer_activity_view.rs:21-50` routes those buttons to harness methods or Settings → Backup/Restore. The synthetic seed can dispatch actual INSERT SQL through the query runtime (`:54-93`); its label says dev/test only, and production requires an explicit confirmation.
- `crates/ui/src/transfer_harness_view.rs:191-291,293-464` confirms the harness boundary: CSV/JSONL/XLSX export uses synthetic rows and temporary files; the CSV “import preview” creates a fixed two-row CSV and previews that fixture; DB→DB uses generated rows and `MemoryTableTarget`, not database endpoints. The masking export also constructs fixed sample rows and writes a temporary file (`:56-154`).
- `crates/ui/src/transfer_state.rs:1-6` stores `Vec<TransferJob>` in transient UI feature state. The visible list is capped by harness code at 20/40 items and has a clear action; no transfer-job persistence path was found in the inspected UI storage/lifecycle or runtime wiring. Do not describe it as durable job history.

## Implemented building blocks and limits

| Area | Source reality at baseline SHA |
|---|---|
| Domain/job model | `crates/core/src/domain/transfer.rs:114-143` has serializable jobs, status, source/target kinds, progress, options, and cancellation state. This is a model, not a persisted job repository. |
| Generic transfer loop | `crates/core/src/application/transfer_service.rs:8-33,103-223` pulls `max_rows` batches, checks cancellation between batches, writes counters into the job, and calls target cleanup on cancellation/failure. It runs synchronously on the caller; progress is mutable job state, not a runtime event stream. Its source contract has a row ceiling only, not a byte budget. |
| Delimited files | `crates/core/src/application/delimited_transfer.rs:32-93,95-164` provides streaming CSV/TSV source and target adapters, with temporary-file publication. The source supports a bounded preview. These adapters are not connected to a live database reader/writer or a user import wizard. |
| JSONL / Excel | `crates/core/src/application/json_excel_transfer.rs:12-70,73-142,152-219` provides JSON Lines source/target and an XLSX target. JSONL streams in rows; XLSX buffers every `TransferRow` in a `Vec` before workbook creation, so that target is not bounded-memory. No XLSX source/reader or JSON-array reader was found in the inspected module. |
| DB-to-DB | `crates/core/src/application/db_transfer.rs:24-69,68-212` contains a row generator and in-memory target used to model policies. The native demo constructs fixed PG→SQLite conversion metadata and runs those in-memory endpoints (`crates/ui/src/transfer_harness_view.rs:367-464`). It is not a live database-to-database path. |
| Native query-result export | `crates/ui/src/query_dialog_surface_view.rs:95-107` offers CSV, TSV, JSON, Markdown, INSERT, and COPY. `crates/ui/src/result_grid_export.rs:43-111` formats the in-memory `UiQueryResult`; `crates/ui/src/query_dialogs_view.rs:100-145` exports only rows held in that result, reports when it is capped, asks before overwrite, and uses atomic publication. The formatter quotes/escapes delimited fields. It is a useful quick export, not a transfer workbench or a streaming table export. |
| Backend `ExportService` | `crates/core/src/application/export_service.rs:46-108` executes a query into a `QueryResult` and renders CSV/JSON/XLSX content. `crates/runtime/src/api.rs:848-865` exposes backup/restore APIs; the export service is runtime-exposed separately, but no native Transfers UI call to it was found. This service also materializes the query result and rendered output. |
| Backup / restore | Settings has a real provider-aware UI and dispatch (`crates/ui/src/settings_backup_view.rs:20-111`, `settings_view.rs:123-184`). Transfers currently deep-links to that Settings section instead of owning the flow (`transfer_activity_view.rs:24-29`). Runtime worker spawns backup/restore work and emits started/terminal status events (`crates/runtime/src/worker.rs:1644-1745`); the native reducer turns those into a transient feedback message (`crates/ui/src/management_events.rs:12-18`). This is not the Transfers job list or row/byte progress. |

### File destination safety gate

The transfer file writers create a `.partial` sibling and rename it to the final path on success. Their constructors do not reject an already-existing final path, and their `cleanup_partial` implementations unconditionally remove both the partial path and `final_path` (`delimited_transfer.rs:104-125,150-164`; `json_excel_transfer.rs:20-35,56-70,162-219`). `TransferService::run` calls `cleanup_partial` on cancellation or failure (`transfer_service.rs:115-160,174-203`) and ignores cleanup errors. Therefore, if a target is constructed with an existing destination and the job cancels/fails before publication, cleanup can delete that pre-existing destination. The current UI exposes these writers only through synthetic temporary-file harnesses, not a user-selected transfer destination; this is a **P2 integration safety gate**, source-derived and not runtime-reproduced here. Before wiring these adapters to real output paths, add explicit no-overwrite/replace semantics and cleanup ownership so a job can only remove files it created. The ordinary Query export path has a separate overwrite-confirmation + atomic-write implementation and does not establish safety for these adapters.

## Provider view

| Workflow | PostgreSQL | SQLite |
|---|---|---|
| File import into a live table | No UI/runtime transfer path found; pending | No UI/runtime transfer path found; pending |
| File/table export from a live database | Native UI can export the currently held query-result rows in common formats. It does not stream a full table or call the backend `ExportService` from Transfers. | Same UI behavior; provider-specific behavior is not exercised by this source-only review. |
| SQL `COPY` result export | The native formatter emits PostgreSQL `COPY … FROM stdin` text. The format option is shown without a visible provider gate (`query_dialog_surface_view.rs:95-107`, `result_grid_export.rs:97-110`). | The same option is shown; generated PostgreSQL syntax is not a SQLite import format. This is a source-level capability/UX mismatch, not a live execution result. |
| Backup / restore | Implemented through `pg_dump` / `psql` / `pg_restore`; local PostgreSQL client tools must be available. Restore is not transactional and may leave a partial database after a mid-run failure (`pg_dump.rs:77-137`; `docs/release/known-limitations.md:317-329`). | Backup uses `VACUUM INTO` with staged publication; restore requires disconnecting the active SQLite connection and validates the staged file with `PRAGMA quick_check` (`sqlite_backup.rs:87-160`; `backup_service.rs:62-96`). |
| Durable transfer jobs | None found for either provider | None found for either provider |

PostgreSQL and SQLite are treated separately here. No live provider runtime evidence was collected. MySQL is not a Phase C supported provider; the shared backup service explicitly rejects it (`crates/core/src/application/backup_service.rs:48-57,85-94`).

## Product-document drift

Several current docs describe an older source baseline and should not be used as evidence for the current native tree without correction:

- `docs/goals/goal-phase-c-transfer.md:10,101-117` says Phase C has not started, has no job model, has no import readers, and has a placeholder rail. The source now has a transfer domain model, a generic transfer loop, file adapters, and a native demo/harness activity. Those are partial foundations, not the real workflows/completion gates specified in that goal.
- `docs/goals/goal-phase-c-transfer.md:108` and `docs/goals/goal-full-product.md:132-134` call native query export unquoted. Current native result export uses delimited escaping, overwrite confirmation, and atomic publication (`result_grid_export.rs:43-61`; `query_dialogs_view.rs:100-145`).
- `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:261-269` marks all import readers and the job model absent, CSV UI unquoted, and the rail placeholder; these rows are stale against the current partial source. Its backup/restore rows remain aligned with the minimal Settings path, subject to runtime qualification.
- `docs/release/known-limitations.md:203-215` states imports are absent, which remains true as an end-to-end database workflow, but the blanket “no reader code” evidence is stale because CSV/TSV and JSONL readers now exist as core adapters. LIM-020 still accurately records the PostgreSQL non-transactional restore consequence and the shipped backup/restore cancellation limitation.

The source does **not** satisfy Phase C Definition of Done: no real import-to-database workflow, no live table/query transfer pipeline, no bounded byte-budget engine, no persistent transfer jobs, no integrated progress/error report, and no per-provider runtime evidence were established in this review (`goal-phase-c-transfer.md:648-683` is the target contract, not proof of implementation).

## Verification

- Source-only review at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- No tests or builds run; no app launched; no PostgreSQL/SQLite runtime evidence collected.
- Source unit tests were inspected in places but not executed; this report does not claim they pass.
