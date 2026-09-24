# Transfers — feature research and recommendation

- Research date: 2026-09-24
- Repository baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: product role, native activity information architecture, workflow requirements, provider boundaries, implementation gaps, and reference-product patterns. No implementation is included.

## Decision

**Keep Transfers as a top-level activity.** The canonical final rail already allocates it as the home for Import, Export, Backup, Restore, Schema Compare, Data Compare, and Migration (`docs/goals/goal-full-product.md:221-258`). The native rail currently has a Transfers item (`crates/ui/src/activity_bar_view.rs:85-96`). This is not the same placement problem as a secondary diagnostic feature: users need a stable entry for cross-cutting data movement and destructive database operations.

**Replace the harness-first content with task entry points.** The current activity is a real surface, but its import/export/DB→DB buttons run fixtures and in-memory mocks; it is not an operational transfer workbench (`docs/design/transfers/transfers-baseline.md`, repository source at the baseline SHA). Avoid putting synthetic test controls in the primary product path. Keep a clearly marked developer harness outside the normal activity if it remains useful.

**Reuse existing query-result export and BackupService; do not conflate them with Transfers completion.** Query export is a safe quick action for rows already loaded in a result. Backup/restore has a real Settings/runtime path. Neither supplies file-to-table import, live table streaming, persistent transfer jobs, or an integrated activity workflow.

## User model

Users arrive with one of four intents:

1. **Bring data in** — choose a file, understand its shape, map fields to a destination, preview consequences, then commit with explicit conflict and transaction behavior.
2. **Take data out** — choose a source (current result, table, selected rows, or query), format and destination, and know whether all rows or only currently fetched rows will be written.
3. **Protect or replace a database** — create a provider-appropriate backup or restore one. Restore is destructive and must identify the target clearly before confirmation.
4. **Inspect work** — find active/recent import, export, backup, and restore jobs, their progress, outcome, destination, and actionable errors.

These are related by the data-movement task, but their semantics differ. Import mutates rows; restore can replace database contents; export writes a file. Keep them in one activity, but give each an explicit flow and safety boundary rather than one generic “run transfer” dialog.

## Recommended activity structure

```text
Transfers
  Overview       Import | Export | Backup | Restore entry cards; recent jobs and capability state
  Import         File → preview → target/mapping → options → review → run → result
  Export         Source → format/destination → preview/options → run → result
  Jobs           Active and persisted history for all four operations
  Related        Schema Compare / Data Compare / Migration links owned by Phase F/later work
```

The Transfers activity can link to Query and Data contexts. Existing quick export from Query should remain convenient, with an “Open in Transfers” path for options or a full-table export. Keep Phase C's explicit non-goal—database-to-database transfer—out of the first transfer implementation; the master IA can still show Phase F compare/migration as related destinations, with phase/capability status rather than fake enabled actions.

### Import flow

Use the Phase C wizard contract as the repository's starting point, not a second design: source/file format and delimiter/header settings; bounded data preview; destination connection/schema/table; column mapping and type overrides; transaction/conflict/error policy; review; run with progress/cancel; result and bounded error-row report (`docs/goals/goal-phase-c-transfer.md:138-156,221-265`). Reopen the file for execution after preview rather than treating a consumed reader cursor as the full job. Never interpolate imported cell values into SQL; writes must use the parameterized mutation path described in the goal (`:302-314`).

Do not silently create a destination table. Make create-table an explicit option with previewed DDL. Show the consequences of whole-file versus per-batch commits before running, including the committed prefix that remains if a per-batch job fails.

### Export flow

Reuse Query's current quick export for the loaded result. Transfers should add explicit source choice (result/table/selection/query), row-scope disclosure, configurable format options, and a streaming path for large sources. Preserve the existing overwrite confirmation and atomic-publish invariant from Query export, but apply a safe destination policy independently to every file adapter. Gate provider-specific SQL formats: the current Query dialog always exposes `COPY` while its formatter emits PostgreSQL `COPY ... FROM stdin` syntax (`crates/ui/src/query_dialog_surface_view.rs:95-107`; `crates/ui/src/result_grid_export.rs:97-110`).

Do not imply that exporting the rows currently held in `UiQueryResult` is a full query export. Current UI intentionally reports when the result contains only the first fetched rows (`crates/ui/src/query_dialogs_view.rs:125-137`).

### Backup and restore

Keep provider behavior explicit and shared with the existing `BackupService`, rather than reimplementing backup in the generic file-transfer engine. A Transfers entry may open an in-activity provider-aware flow or navigate to the existing Settings panel, but it should not leave users with a dead-end/settings detour from the main data task.

Before restore, name the connection/database and environment, identify input file/format, state the destructive scope, and require a strong confirmation for production. Warn SQLite users before dispatch that the active connection must be disconnected. Show PostgreSQL client-tool prerequisites. Do not promise atomic restore or cancellation where the provider path does not provide it.

### Jobs and background behavior

A job list is part of the feature, not a list of UI test results. It should include Import, Export, Backup, and Restore with started/finished time, provider/connection, source/target summary, counters, terminal status, and error details. Persist records through the meta store with bounded retention; redact full SQL, row data, credentials, and unnecessary absolute paths as the Phase C goal requires (`goal-phase-c-transfer.md:205-219,278-300`).

The transfer engine should execute off the immediate egui action path. Progress should be event-driven and rate-limited; cancellation must be wired to the runtime cancellation map and report exact committed/rolled-back state. The current core engine only updates a mutable in-memory job synchronously; its API caps rows but not bytes (`crates/core/src/application/transfer_service.rs:8-33,103-223`). The current backup worker reports started/terminal events, but the native reducer sends them only to transient feedback (`crates/runtime/src/worker.rs:1644-1745`; `crates/ui/src/management_events.rs:12-18`). Do not count either as persistent transfer-job behavior.

## Provider contract

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Import CSV / JSON / XLSX into a table | Pending end-to-end UI/runtime path. Generic file readers are not database adapters. | Pending end-to-end UI/runtime path. Do not infer support from PostgreSQL. |
| Export table / full query stream | Pending integrated data source and streaming UI path. Current quick export covers the loaded result only. | Pending integrated data source and streaming UI path. Current quick export covers the loaded result only. |
| SQL `COPY` | PostgreSQL-specific format; offer only for this provider and describe file/script semantics. | Unsupported as a SQLite import command; hide or explain rather than emitting PostgreSQL syntax. |
| Backup | Supported through external `pg_dump`; client tools required on PATH. | Supported through `VACUUM INTO`, with staged artifact validation/publication. |
| Restore | Supported through `psql`/`pg_restore`, but current restore is not transactional; partial state is possible on failure. | Supported after disconnecting the active DB; staged artifact is checked before publication. |
| Transfer job history | Not implemented | Not implemented |

The provider matrix must be updated with separate evidence for each provider. Source parity is not runtime proof. Existing provider-specific backup details are in `crates/core/src/application/backup_service.rs:39-109`, `crates/infrastructure/src/backup/pg_dump.rs:109-137`, and `crates/infrastructure/src/backup/sqlite_backup.rs:87-160` at the baseline SHA.

## Reference-product observations

- [DBeaver — Data transfer](https://dbeaver.com/docs/dbeaver/Data-transfer/) organizes import/export and table-to-table transfer as distinct operations; lists supported formats; exposes format-specific extraction/encoding/null/delimiter settings; documents background execution and saved export/import configurations. Useful pattern: source/target and options are explicit, with reusable task configuration. This is not a mandate to implement DB→DB in Phase C.
- [DBeaver — Backup and restore](https://dbeaver.com/docs/dbeaver/Backup-Restore/) states that backup/restore uses each database's native client tools, distinct from data transfer. Useful boundary: database backup is not just another row export. It describes provider-specific format and client settings and saving operations as tasks.
- [DataGrip — Import](https://www.jetbrains.com/help/datagrip/import-data.html) documents file/table/query-result imports through a mapping tree, source-specific settings, target table/schema selection, and a preview pane for source data and generated DDL. Useful pattern: keep mapping, source configuration, target selection, and preview in one inspectable import workflow.

Reference pages were read on 2026-09-24. Their behavior is evidence about those products only; it does not establish DB Pro provider capabilities or runtime evidence.

## Prioritized open work

1. **P1 product gap — real import into PostgreSQL and SQLite.** File readers and preview helpers are not an import service; there is no live database target, user workflow, or runtime evidence. This is an unimplemented Phase C outcome, not a regression claimed by this research.
2. **P1 product gap — export beyond the loaded query result.** There is no integrated streaming table/query source in Transfers. The quick export accurately exports what is in memory but is not an all-rows workbench.
3. **P2 integration safety risk — transfer file adapters can remove an existing final path during failure/cancellation cleanup.** Source-derived at this SHA (`delimited_transfer.rs:104-164`, `json_excel_transfer.rs:20-70`, `transfer_service.rs:115-160`); no runtime reproduction was performed. These adapters are currently used by synthetic harnesses only. Establish no-overwrite/replace policy and cleanup ownership before accepting user destinations.
4. **P2 product gap — durable job history and integrated progress/cancel.** Current jobs are transient UI harness state; no transfer jobs survive restart. Progress must reflect actual provider commit/rollback behavior.
5. **P2 scale gap — byte-bounded memory and XLSX handling.** The generic transfer loop bounds rows only; the XLSX target buffers the entire dataset. Reject large-run promises until memory behavior is measured and bounded or a file-size/row limit is explicit.
6. **P2 provider UX — gate PostgreSQL COPY output.** The native export dialog displays COPY without a driver condition; generated syntax is PostgreSQL-specific.
7. **P2 documentation drift — refresh Phase C status and capability tables.** Current docs call the transfer rail a placeholder, say the domain job model/readers are absent, and say native query export is unquoted. The current tree has partial groundwork and a real but harness-oriented activity; it still misses the feature's end-to-end gates.

## Recommended decision boundary

Retain the rail and use Phase C as the feature owner. Before implementation planning, update the capability baseline and phase-goal “current implementation” section against the exact code SHA; explicitly distinguish demo scaffolding from shipped data movement. Split implementation by observable outcomes, but do not declare Phase C complete until import, large-result export, provider-specific backup/restore monitoring, durable jobs, and separate PostgreSQL/SQLite runtime evidence meet the goal's Definition of Done (`goal-phase-c-transfer.md:648-683`).

## Research limits

No feature plan/status transition was made. No code or tests were changed/run, and no native UI, PostgreSQL, or SQLite runtime scenario was exercised. The P1/P2 entries are product gaps or source-level integration risks, not a claim that these behaviors were newly introduced at this SHA.
