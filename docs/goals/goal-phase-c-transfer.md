# DB Pro — Phase C Goal: Data Transfer (Import / Export / Backup / Restore)

- Doc ID: `GOAL-PHASE-C`
- Phase: C — Data Transfer
- Release target: v0.3 (all five milestones)
- Priority: P1 (C01, C02, C05), P2 (C03, C04)
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: §4.6 meta-store migration pattern (master goal); reuses `ExportService` and
  `BackupEngine`; uses the `DataGrid` component for preview.
- Status: PLANNING. No implementation has started.
- Companions: `docs/goals/goal-phase-d-monitoring.md` (job/progress patterns),
  `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §9/§10, `docs/release/known-limitations.md`
  (LIM-012 import deferred, LIM-015 pg_dump on PATH).

---

## 1. Problem

DB Pro can move data **out** in the backend and almost nothing in practice; it cannot move
data **in** at all.

Observed facts:

1. **There is no import code anywhere.** No CSV/JSON/XLSX reader, no import service, no import
   command, no reader dependency in any crate. `docs/release/provider-capability-matrix.md`
   records `Import | DEFERRED` for both providers, and `LIM-012` states the same. The
   Transfers activity in the native UI is a `COMING SOON` placeholder
   (`crates/ui/src/navigation_view.rs:610-615`).
2. **The export engine exists but is unreachable from the native UI.**
   `ExportService::export_csv/export_json/export_excel`
   (`crates/core/src/application/export_service.rs:65-249`) implement quoting, JSON
   duplicate-column rejection, non-finite-float rejection, and lossless handling of integers
   beyond `2^53` in XLSX. The native UI does not call them. Instead
   `crates/ui/src/query_view.rs:1357-1389` writes a file with a hand-rolled writer that joins
   columns with `,` or `\t` and **performs no quoting or escaping** — a value containing a
   comma, quote, or newline corrupts the file. The capability matrix already labels the export
   rows `BACKEND_ONLY` and notes "fix native CSV quoting".
3. **There is no streaming or progress anywhere.** Exports materialize a bounded `QueryResult`
   and write it; backup runs `pg_dump`/`psql`/`pg_restore` as a child process with a timeout
   and no progress reporting; the only "progress" is a coarse started/completed/failed event
   from the legacy Tauri wrapper (`crates/tauri-app/src/commands/backup.rs:10-15`), which the
   native path does not use. There is no cancel for a running backup (the runtime has a
   `CancelMap` for backup requests, but `RuntimeCommand::CancelOperation`, its only trigger,
   is never constructed — see `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §16 and the master
   goal's audit).
4. **Backup/restore UX is minimal.** The Settings sidebar exposes path selection plus run for
   both providers (`navigation_view.rs:994-1062`), gated on the `features.backup` capability;
   restore has an overwrite confirmation. There is no progress, no format choice, no
   post-restore verification beyond what the engines already do, and no visibility of the
   `pg_dump`-on-PATH requirement (`LIM-015`) until the command fails.
5. **There is no transfer job model.** Nothing records that an import/export/backup happened,
   whether it succeeded, how many rows it wrote, or where the errors were. A failed import
   leaves nothing to diagnose beyond a single error string.

Consequences for users: importing a CSV into a table requires an external tool; exporting a
result set from the native UI can silently produce a malformed file; a long backup cannot be
seen or stopped; and no record survives a failure.

**This phase makes data movement a first-class, streaming, observable, cancellable
operation.**

## 2. Scope

### 2.1 In scope

| Capability | Milestone |
|---|---|
| Streaming transfer engine: chunked read → batched write, transaction policy, backpressure, bounded memory, progress, cancellation, error-row collection | C01 |
| Transfer job records persisted in the meta store with status, counts, timing, and error summary | C01 |
| CSV import wizard (encoding, delimiter/quote detection, header, preview, mapping, type inference/coercion, null/default policy, batch size, transaction mode, conflict mode, progress, cancel, error report) | C02 |
| Excel (`.xlsx`) import with sheet selection, sharing the C02 pipeline | C03 |
| JSON / NDJSON import with nested-value policy, sharing the C02 pipeline | C04 |
| Export workbench: wire `ExportService` (CSV/TSV/JSON/XLSX) to real sources; delete the local unquoted writer; add SQL `INSERT` generation and PostgreSQL `COPY` export | C05 |
| Transfers activity: import/export/backup/restore entries plus a live job list (placeholder removed) | C05 |
| Backup/restore UX completion: progress, cancel, format choice where the engine supports it, post-run verification summary | C05 |

### 2.2 Explicitly out of scope (later phases)

- Database-to-database transfer (structure + data) — Later (master goal §6.6).
- Schema inference from a file into a full migration — Phase F.
- Scheduled/recurring transfers — Phase J.
- Compressed archives, cloud destinations, resumable transfers.
- Data compare beyond what Phase F provides.

## 3. Non-goals

- **No full-dataset materialization.** An import or export must never hold the entire dataset
  in memory. A design that reads the whole file or builds one giant `QueryResult` is rejected
  in review regardless of how well it performs on a small fixture.
- **No silent partial writes.** The default transaction policy is whole-file; a per-batch
  policy is opt-in, explicitly labeled, and reports exactly how many rows committed.
- **No automatic type guessing presented as certainty.** Inferred types are shown with
  confidence and are always user-overridable.
- **No "import into a new table" as an implicit schema migration.** Creating a destination
  table from a file is supported as an explicit, previewed action (it generates DDL), and it
  is not a substitute for Phase A's table editor.
- **No overwriting an output file silently** (retain the existing `create_new` behavior in the
  backup engines and apply the same rule to exports).
- **No new grid implementation.** Import preview uses the shared `DataGrid`.

## 4. Current implementation (code-verified baseline)

| Element | Reality | Evidence |
|---|---|---|
| Import (any format) | **Absent** | no reader code or dependency in `crates/`; `docs/release/provider-capability-matrix.md` records IMPORT as DEFERRED; `LIM-012` |
| Export engine | Works, backend only | `core/src/application/export_service.rs:65-249` (`export_csv`, `export_json`, `export_excel`, `validate_json_column_names:170`, `excel_column_index:159`, `excel_row_index:163`, `2^53` guard `:183-187`) |
| Export policy | Read-only enforced | `export_service.rs:40-63` (`reject_multi_statement`, policy from `config.readonly`, registry handle required, `result.validate()`) |
| Native export (UI) | **Unquoted local writer** | `ui/src/query_view.rs:1357-1389` (`std::fs::write`, delimiter join, no escaping); reached from the Results pane button (`query_view.rs:418-420`) and `PaletteAction::ExportResults` (`palette_view.rs:138,224,270-277`) |
| Clipboard copy | Rich but separate | `result_grid_view.rs:1721` row CSV, `:1751` all CSV, `:1783` all JSON, `:1648` INSERT statements (staged-copy gap noted in the capability matrix) |
| Streaming | **Absent** | exports materialize a bounded `QueryResult`; `BackupEngine` (`core/src/ports/backup_engine.rs:8-11`) has no progress parameter or sink |
| Backup — PostgreSQL | Works via external binaries | `infrastructure/backup/pg_dump.rs:79-206` (`pg_dump` with `-h -p -U -d -f`, plain/custom format, `-n`/`-t` filters, `PGPASSWORD`, `kill_on_drop`, timeout → `QueryTimeout`, partial output deleted on failure, `create_new` reserve) |
| Backup — SQLite | Works, different mechanism | `infrastructure/backup/sqlite_backup.rs:60-157` (`VACUUM INTO` into a temp file, `busy_timeout(5s)`, atomic `hard_link` publish; restore validates with `PRAGMA quick_check` in read-only mode then renames) |
| Backup/restore UI | Minimal | `ui/src/navigation_view.rs:940-1062` (path pick + run, gated on `features.backup`), overwrite confirmation on restore |
| Backup progress/cancel | **Absent** in the native path | `RuntimeCommand::CancelOperation` (the only cancel trigger for the backup `CancelMap`) is never constructed |
| Restore guards | Present | `core/src/application/backup_service.rs:61-71` (read-only blocks restore; SQLite requires the target connection to be disconnected) |
| Transfer job model | **Absent** | no job entity, no meta-store table, no history |
| Meta store | Ready for extension | `infrastructure/meta/schema.rs` (9 tables), `migration.rs` (`LATEST_VERSION = 2`, idempotent `migrate`) |

## 5. UX

### 5.1 Transfer activity

```text
TRANSFER
  Import                 → wizard (CSV | Excel | JSON)
  Export                 → workbench (result | table | selection | query)
  Backup                 → database backup (provider-aware)
  Restore                → database restore (provider-aware, destructive)
  Jobs                   → live + historical transfer jobs
  Schema Compare         → Phase F entry (present but disabled until F01)
  Data Compare           → disabled until a later phase; states the reason
  Migration              → Phase F entry
```

The `COMING SOON` placeholder is removed by C05 when the first real surface lands; until then
the placeholder text must name the milestone (no silent dead icon).

### 5.2 Import wizard (shared shell for CSV/Excel/JSON)

```text
Step 1  Source      file picker, detected format, encoding, delimiter/quote (CSV), sheet (XLSX),
                    JSON shape (array | NDJSON), nest policy
Step 2  Preview     first N rows (virtualized grid), detected column types with confidence,
                    per-column override, "row 1 is header" toggle
Step 3  Target      connection → schema → table (picker), or "Create new table" (generates DDL
                    preview), column mapping (source → target, skip, constant, default), NULL
                    representation ("", \N, NULL, custom), default-on-missing behavior
Step 4  Options     batch size, transaction mode (whole file | per batch), conflict mode
                    (insert | skip existing | update by key), on-error (abort | skip row),
                    timeout
Step 5  Review      generated INSERT statement template with placeholders, estimated row count,
                    destination columns, warnings (type coercion, truncation, NOT NULL violations
                    detected in the preview sample)
Step 6  Run         progress (rows read/written/skipped/failed), throughput, cancel, elapsed
Step 7  Result      committed/skipped/failed counts, duration, error-row report (downloadable),
                    job record link
```

Rules:

- Steps 1–5 never write to the database. The wizard can be closed at any point with no side
  effects.
- The preview is a real typed grid (`DataGrid`) with per-cell type errors highlighted, using
  the same rendering as the table editor.
- Column mapping auto-matches by name first, then by position, and always shows what it
  guessed.
- "Create new table" is an explicit choice that produces DDL through Phase A's preview →
  confirm → apply path; it is not executed by the import wizard directly.

### 5.3 Export workbench

```text
Source      ○ Current result set   ○ Table (with current filters/sorts)   ○ Selected rows
            ○ Query (SQL editor content)   ○ Whole schema (table list → multi-file or single)
Format      ○ CSV  ○ TSV  ○ Excel (.xlsx)  ○ JSON  ○ SQL INSERT  ○ SQL COPY (PostgreSQL only)
Options     header row, delimiter, quote/escape, NULL representation, encoding (UTF-8/BOM),
            date/number format, one file per table (for schema export), include CREATE TABLE DDL
Destination file picker (never overwrite silently)  |  [ Preview first 100 rows ]  [ Export ]
```

- The unquoted local writer is deleted in C05; all export paths go through `ExportService`.
- SQL INSERT export emits replayable statements with correct literal escaping, batched into
  multi-row `VALUES` where the target dialect allows it, and a transaction wrapper option.
- `COPY` export is shown only for PostgreSQL connections (capability-gated) and emits a
  `COPY … FROM STDIN` script, not a server-side file write.
- Large exports stream; the progress bar reports rows written and bytes.

### 5.4 Backup / restore

- **Backup**: choose scope (whole database | schemas | tables), format where supported
  (PostgreSQL: plain SQL vs custom; SQLite: single mechanism), destination, and whether to
  include data and/or schema. Progress is reported per phase (spawn → run → publish) with an
  elapsed timer and a Cancel action.
- **Restore**: choose source file, target connection, and behavior (drop-and-recreate vs
  merge where meaningful). The confirmation payload lists target, environment, file, size, and
  the destructive warning; Production requires the typed connection name. SQLite restore
  requires the connection to be disconnected — the UI must state this **before** the attempt,
  not as an error afterwards.
- PostgreSQL backup/restore checks for `pg_dump`/`pg_restore`/`psql` availability up front and
  reports a clear "not found on PATH" error with the platform prerequisite link
  (`LIM-015`, `docs/release/platform-prerequisites.md`).
- After a restore completes, the UI offers to re-introspect the connection and reports the
  object counts it sees, so the user gets verification rather than a green checkmark.

### 5.5 Job list

Every import/export/backup/restore creates a job record visible in Transfer → Jobs:

```text
Started             Kind      Target                       Rows        Result     Duration
14:02:11  Import    sales.orders (prod)          1,204,551    Committed   00:42
14:10:03  Export    query → orders.csv             204,551      Written   00:09
13:55:40  Backup    local-pg (development)              —       Failed: pg_dump not found
```

Jobs show status (`Running`/`Completed`/`Failed`/`Cancelled`), a cancel action while running, a
detail view with the error report, and a link to the destination path. Records survive restart.
Retention is bounded (for example, keep the most recent N per connection and prune older rows)
so the meta store cannot grow unbounded.

## 6. Architecture

### 6.1 The streaming pipeline

```text
                    ┌────────────────────────────────────────────┐
 TransferSource ──▶ │  TransferEngine (chunked, bounded memory)  │ ──▶ TransferSink
 (File | Query |    │  • chunk size (rows + bytes)               │    (Table | File)
  Table | Stdin)    │  • transaction policy                      │
                    │  • progress emitter (rate-limited)         │
                    │  • cancellation check between chunks       │
                    │  • error-row collector (bounded ring)      │
                    └────────────────────────────────────────────┘
```

Contract:

```rust
#[async_trait]
pub trait TransferSource: Send {
    async fn next_chunk(&mut self, budget: ChunkBudget) -> Result<Option<Chunk>, TransferError>;
    fn schema(&self) -> SourceSchema;             // column names + inferred types
    fn estimated_rows(&self) -> Option<u64>;
}

#[async_trait]
pub trait TransferSink: Send {
    async fn begin(&mut self, plan: &SinkPlan) -> Result<(), TransferError>;
    async fn write_chunk(&mut self, chunk: &Chunk) -> Result<ChunkOutcome, TransferError>;
    async fn commit(&mut self) -> Result<(), TransferError>;
    async fn rollback(&mut self) -> Result<(), TransferError>;
}
```

Rules that make the design honest:

- `ChunkBudget` carries both a row cap and a byte cap; the source must respect both, so a
  table with very wide rows cannot blow memory.
- The engine never accumulates more than one chunk plus the error ring in memory.
- Cancellation is checked between chunks and between batches; the sink performs a deterministic
  rollback or reports the committed prefix (per transaction policy).
- Progress events are emitted at most ~10 per second and always carry
  `(rows_read, rows_written, rows_skipped, rows_failed, bytes, elapsed)`.
- Error rows are collected into a bounded ring (for example the first 1,000 failures plus a
  count) and the full report is streamed to a file rather than held in memory.

### 6.2 Transaction policy

| Policy | Behavior | When offered |
|---|---|---|
| `WholeFile` (default) | One transaction around all batches; abort → full rollback | Default; recommended for correctness |
| `PerBatch` | Commit each batch; abort → committed prefix remains | Offered with an explicit warning and a committed-count display |
| `None` (dry run) | Validate and report without writing | Always offered ("Validate only") |

The engine reports, on failure, exactly which mode was used, how many rows committed, and the
first failing row with its line number (CSV/JSON) or cell reference (XLSX).

### 6.3 Job model

```rust
pub struct TransferJob {
    pub id: TransferJobId,
    pub connection_id: ConnectionId,
    pub kind: TransferKind,          // Import | Export | Backup | Restore
    pub source: String,              // redacted description (file name, table, query digest)
    pub target: String,              // redacted description
    pub options_digest: String,      // hash of the option set (no values)
    pub status: TransferStatus,      // Queued | Running | Completed | Failed | Cancelled
    pub counters: TransferCounters,  // read/written/skipped/failed/bytes
    pub started_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
    pub error_summary: Option<String>,
    pub error_report_path: Option<PathBuf>,
}
```

Persisted in the meta store (`transfer_jobs` table + migration). The `options_digest` exists so
a job can be described in the UI and in logs **without** storing the file path verbatim or any
row values (secrets and data must not leak; see master goal §4.6 and
`docs/architecture/security-boundaries.md`).

### 6.4 Layering

- `TransferService` (application) depends on `TransferSource`/`TransferSink` ports and on
  `TableDataService`/`QueryService` for the sink's SQL. It never builds SQL in the UI and never
  opens a connection outside the registry.
- Readers (CSV/XLSX/JSON) are implementations of `TransferSource` living in
  `crates/infrastructure/src/transfer/` (new module) so that `crates/core` stays
  dependency-light and the file-format concerns stay in infrastructure.
- Writes use the existing parameterized mutation path (`build_insert`/`build_update` in
  `crates/core/src/application/sql_builder.rs`) so values are always bound, never interpolated.
  Batch writes use multi-row `INSERT` with generated placeholders (dialect-aware).
- Progress and cancellation flow through the existing worker maps; no second cancellation
  registry.

## 7. Domain model

New module `crates/core/src/domain/transfer.rs`:

```rust
pub enum TransferKind { Import, Export, Backup, Restore }

pub enum TransferStatus { Queued, Running, Completed, Failed, Cancelled }

pub struct TransferCounters { pub rows_read: u64, pub rows_written: u64, pub rows_skipped: u64,
                              pub rows_failed: u64, pub bytes: u64 }

pub struct ChunkBudget { pub max_rows: usize, pub max_bytes: usize }

pub struct Chunk { pub rows: Vec<Vec<CellValue>>, pub first_row_number: u64 }

pub enum SourceSchema { Known(Vec<ColumnMeta>), Inferred(Vec<InferredColumn>), Unknown }

pub struct InferredColumn { pub name: String, pub inferred: ColumnTypeGuess, pub confidence: f32,
                            pub sample_errors: u32 }

pub enum ConflictMode { Insert, SkipExisting, UpdateByKey { key_columns: Vec<String> } }

pub enum TransactionPolicy { WholeFile, PerBatch, ValidateOnly }

pub enum OnError { Abort, SkipRow }

pub struct ImportOptions { pub encoding: Option<String>, pub delimiter: Option<char>,
    pub quote: Option<char>, pub has_header: bool, pub sheet: Option<String>,
    pub json_shape: JsonShape, pub nested_policy: NestedPolicy,
    pub mapping: Vec<ColumnMapping>, pub null_literals: Vec<String>,
    pub conflict: ConflictMode, pub transaction: TransactionPolicy,
    pub on_error: OnError, pub batch_size: usize, pub timeout_ms: Option<u64> }

pub struct ExportOptions { pub format: ExportFormat, pub header: bool, pub delimiter: char,
    pub quote: char, pub null_repr: String, pub encoding_bom: bool,
    pub include_ddl: bool, pub multi_file: bool, pub transaction_wrapper: bool }
```

`ExportFormat` extends the existing `ExportService` formats with `SqlInsert` and `SqlCopy`
(the latter PostgreSQL-only).

## 8. APIs, services, ports

### 8.1 Ports

| Port | Methods | Implementations |
|---|---|---|
| `TransferSource` (new) | `next_chunk`, `schema`, `estimated_rows` | `CsvSource`, `XlsxSource`, `JsonSource`, `QuerySource` (wraps `QueryService`), `TableSource` |
| `TransferSink` (new) | `begin`, `write_chunk`, `commit`, `rollback` | `TableSink` (parameterized `INSERT`/`UPDATE`), `FileSink` (delegates to `ExportService`) |
| `TransferJobRepository` (new) | `save`, `update_status`, `list(connection_id, limit)`, `get`, `prune` | `SQLiteMetaStore` |
| `ExportService` (extend) | add `export_sql_insert`, `export_sql_copy`, and streaming variants that accept a `RowStream` instead of a materialized `QueryResult` | existing |

### 8.2 Service

```rust
impl TransferService {
    // Import
    pub async fn inspect_source(&self, spec: SourceSpec) -> Result<SourcePreview, TransferError>;
    pub async fn plan_import(&self, spec: SourceSpec, target: ImportTarget, options: ImportOptions)
        -> Result<ImportPlan, TransferError>;              // no writes; returns mapping + SQL template + warnings
    pub async fn run_import(&self, plan: ImportPlan, progress: ProgressSink) -> Result<TransferJob, TransferError>;

    // Export
    pub async fn plan_export(&self, source: ExportSource, options: ExportOptions)
        -> Result<ExportPlan, TransferError>;
    pub async fn run_export(&self, plan: ExportPlan, progress: ProgressSink) -> Result<TransferJob, TransferError>;

    // Backup / restore (wrapping the existing BackupService, adding progress + jobs)
    pub async fn run_backup(&self, options: BackupOptions, progress: ProgressSink) -> Result<TransferJob, TransferError>;
    pub async fn run_restore(&self, options: RestoreOptions, progress: ProgressSink) -> Result<TransferJob, TransferError>;

    // Jobs
    pub async fn list_jobs(&self, connection_id: ConnectionId, limit: usize) -> Result<Vec<TransferJob>, DbError>;
    pub async fn cancel_job(&self, job_id: TransferJobId) -> Result<(), DbError>;
}
```

`plan_import`/`plan_export` are pure planning steps: they read metadata and the source header,
produce the mapping, the generated statement template, and warnings — and they never write.

### 8.3 Runtime/UI wiring

```text
UiCommand::InspectTransferSource { request_id, spec }
UiCommand::PlanImport            { request_id, spec, target, options }
UiCommand::RunImport             { request_id, plan }
UiCommand::PlanExport            { request_id, source, options }
UiCommand::RunExport             { request_id, plan }
UiCommand::RunBackup             { request_id, options }
UiCommand::RunRestore            { request_id, options }
UiCommand::ListTransferJobs      { request_id, connection_id, limit }
UiCommand::CancelTransferJob     { request_id, job_id }

UiEvent::TransferSourceInspected { request_id, preview }
UiEvent::ImportPlanned           { request_id, plan }
UiEvent::ExportPlanned           { request_id, plan }
UiEvent::TransferProgress        { request_id, job_id, counters, elapsed_ms }   // rate-limited
UiEvent::TransferCompleted       { request_id, job }
UiEvent::TransferFailed          { request_id, job_id, error, error_report_path }
UiEvent::TransferJobsLoaded      { request_id, jobs }
```

`TransferProgress` is deliberately a distinct event from `RuntimeEvent::OperationProgress` only
if the payload genuinely differs; otherwise extend `OperationProgress` with counters and reuse
it. Decide in C01 and document the choice — do not ship two progress mechanisms.

### 8.4 Backup/restore integration

- Extend `BackupEngine` (`crates/core/src/ports/backup_engine.rs:8-11`) with a progress-aware
  variant, or wrap it: the engine reports phase transitions and byte counts; `PgDumpEngine` can
  read the child's stderr and file growth, `SqliteBackupEngine` reports `VACUUM INTO`
  start/finish. `kill_on_drop` + the timeout already exist; add an explicit cancel that kills
  the child and deletes the partial output (the current code does this on failure — reuse it).
- Restore must surface the SQLite "connection must be disconnected" precondition
  (`backup_service.rs:67-71`) in the UI before the attempt.
- Format choice: PostgreSQL already supports plain/custom in the engine
  (`pg_dump.rs:79-206`); expose it. SQLite has one mechanism — do not invent a format selector.

## 9. Provider behavior

### 9.1 PostgreSQL

| Aspect | Behavior |
|---|---|
| Insert batching | Multi-row `INSERT` with `$n` placeholders; batch size configurable; `RETURNING` not required |
| Conflict mode | `SkipExisting` via `ON CONFLICT DO NOTHING`; `UpdateByKey` via `ON CONFLICT (keys) DO UPDATE SET …` — both require a unique index; the wizard validates that one exists and reports the reason when it does not |
| Identity/`DEFAULT` | A column can be mapped to "DEFAULT" (omit from the insert list) with an explicit mapping option |
| COPY export | `COPY … FROM STDIN` script generation; server-side `COPY … TO file` is never used (requires superuser and writes on the server) |
| Backup | `pg_dump`/`pg_restore`/`psql` from PATH (`LIM-015`); check availability before starting; `PGPASSWORD` env, never argv |
| SSH | Backup honors the SSH tunnel (`pg_dump.rs:22-40`); import/export over an SSH-tunneled connection work through the existing tunneled pool |
| Large imports | `COPY`-based import is a candidate optimization but is **not** in scope for C02 (parameterized batches first); record it as a follow-up if measured throughput is insufficient |

### 9.2 SQLite

| Aspect | Behavior |
|---|---|
| Insert batching | Multi-row `INSERT` with `?` placeholders; `INSERT OR IGNORE` for `SkipExisting`; `INSERT … ON CONFLICT(key) DO UPDATE` for `UpdateByKey` |
| Transactions | Explicit `BEGIN`/`COMMIT` around batches; whole-file mode uses one transaction (SQLite handles this well) |
| JSON columns | Values are stored as `TEXT`; the import maps objects/arrays to JSON text with an explicit statement of that behavior (`LIM-013`) |
| Date/time | Stored as `TEXT`; the wizard offers ISO-8601 normalization with an explicit "no native date type" note |
| Backup | `VACUUM INTO` + atomic publish — **not** a file copy (correct the release-doc wording, master goal §2.5 C11) |
| Restore | Requires the target connection to be disconnected; validation via `PRAGMA quick_check` |
| COPY / formats | Not available; the export workbench hides `SQL COPY` for SQLite |

### 9.3 Cross-provider

- A transfer targets exactly one provider; there is no cross-provider transfer in this phase.
- Format/capability differences are surfaced through `DatabaseCapabilities` and the export
  format list, never by a driver check in a view.
- The `ExportService` guards (duplicate JSON column names, non-finite floats, `2^53`
  integers) are provider-neutral and must be preserved in the streaming rewrite — losing them
  would be a regression.

## 10. UI structure

### 10.1 Components

| Need | Component | Status |
|---|---|---|
| Wizard shell (steps, back/next, validation) | `FormEditor` + a new `TransferWizard` composition | new composition over existing primitives |
| File/folder picking | `rfd` dialogs through the native command thread (pattern exists: `native-app/src/main.rs:57-87`) | reuse |
| Data preview grid | `DataGrid` (`result_grid.rs`, `result_grid_view.rs`) | reuse |
| Column mapping table | `ObjectList`/`Table` + `Select` per row | new composition |
| Progress | `ProgressView` (`components/feedback.rs`, `ProgressRing`) | reuse |
| Error report | `DataGrid` + export-to-file action | reuse |
| Job list | `ObjectList` with status badges (`components/badge.rs`) | new composition |
| Confirmations | `ConfirmationDialog` with the master-goal §10.4 payload | reuse |
| Activity log | `ActivityLog` | promote from gallery |

### 10.2 Screens

1. **Transfer activity** landing: five entries + job list summary.
2. **Import wizard** (7 steps, §5.2).
3. **Export workbench** (§5.3).
4. **Backup dialog** (scope/format/destination) and **Restore dialog** (file/target/behavior).
5. **Job detail**: counters, timings, error report link, cancel while running.
6. **Result grid export menu** rewired to the workbench (fixes the unquoted writer).
7. **Schema header "Export…"** action for a whole table with current filters/sorts.

### 10.3 Interaction rules

- The wizard must be fully keyboard-navigable; `⌘⏎` advances when the current step is valid.
- Leaving the wizard during the Run step requires an explicit confirmation (the transfer may
  be mid-flight) — the preferred behavior is to keep the wizard open and let the job continue
  in the job list if the user navigates away.
- Progress display is honest: before the first progress event it shows "starting", never a fake
  percentage. Indeterminate phases (spawn, validate) use the indeterminate progress style.
- Error reports are downloadable and also viewable in-app; the first N error rows are shown
  immediately.
- Rows that failed must never be silently dropped — the result step always reports the four
  counters (read/written/skipped/failed).

## 11. Safety model

| Operation | Class | Notes |
|---|---|---|
| Export to file | `ReadOnly` | Reads pass the existing export policy; writes to the filesystem are guarded by the "never overwrite silently" rule |
| Import into an existing table | `Mutation + LongRunning` | Preview + confirmation with the §10.4 payload; whole-file transaction default; read-only connections blocked |
| Import creating a new table | `Mutation` (DDL) **then** `Mutation` (data) | Table creation goes through Phase A's preview/apply; data import is a separate confirmed step |
| Restore | `Destructive + LongRunning` | Overwrite confirmation with file/target/size/environment; Production requires typed connection name; SQLite requires a disconnected target |
| Backup | `LongRunning` (read-only in effect) | No confirmation needed beyond choosing a destination; must not overwrite an existing file |
| Cancel a running transfer | n/a | Must leave a deterministic state and report committed counts |
| Schema export including DDL | `ReadOnly` | Generated DDL is data, not execution |

Additional rules:

- No import may violate NOT NULL/type constraints silently: violations are either coerced
  with a recorded decision or reported as failed rows.
- Bound values only. The generated statement template contains placeholders; a literal value in
  the template is a review-blocking defect.
- The `pg_dump` password path (env var, not argv) and the existing secret handling must be
  preserved.
- Job records must not contain row values, file contents, or credentials. Source/target
  descriptions are redacted (file name only, query digest rather than full SQL).

## 12. Testing strategy

### 12.1 Unit

- Readers: BOM, CRLF/LF/CR, quoted fields containing delimiters/newlines/quotes, escaped
  quotes, ragged rows, empty file, header-only file, duplicate header names, unknown encoding,
  multi-byte characters split across chunk boundaries, very wide rows.
- XLSX: shared strings, inline strings, numeric/boolean/date cells, formulas with cached
  values, merged cells, empty sheets, multiple sheets, leading/trailing empty rows.
- JSON: array, NDJSON, nested objects/arrays, `null` values, type mixing within a field,
  malformed input with byte offsets, very large single object.
- Type inference: confidence calculation, per-column override, coercion errors
  (`"abc"` → integer), overflow (`2^53` boundaries and beyond), decimal precision retention.
- Pipeline: chunk budget enforcement (rows and bytes), backpressure, error-ring bound,
  progress rate limiting, cancellation between chunks and between batches.
- Statement generation: placeholder correctness per dialect, `ON CONFLICT` variants, DEFAULT
  omission, identifier quoting for reserved/odd table and column names.
- Job repository: save/update/list/prune, migration idempotency.

### 12.2 Service / integration

- SQLite (always-run): CSV import into a fixture table (whole-file and per-batch), cancel
  mid-import and assert the committed prefix matches the reported count, conflict modes,
  error-row report, export round-trip (export → import → compare row counts and values).
- PostgreSQL (live, ignored suite): the same matrix plus multi-row batching behavior,
  `ON CONFLICT` paths, `COPY` export script validity, backup/restore live runs (existing
  `ssh_backup_runtime_verification.rs` pattern extended).
- Streaming assertions: run a 100k-row transfer under a memory cap assertion (for example
  instrument the engine's peak chunk allocation) to prove the design does not materialize the
  dataset.
- Backup/restore: `pg_dump` missing from PATH produces the documented error; SQLite restore
  with an active connection produces the documented precondition error; cancel kills the child
  and removes the partial output.

### 12.3 UI

- Wizard step validation, back/forward, and the guarantee that closing before Run writes
  nothing (assert no command emitted).
- Mapping UI: auto-match, override, skip column, constant value, DEFAULT mapping.
- Progress rendering: indeterminate before the first event; rate-limited updates; cancel
  during Run leaves an explicit final state.
- Job list: statuses, cancel, detail, error report download, retention behavior.
- Export workbench: format list per provider capability; `SQL COPY` absent on SQLite; the
  result-grid export now uses the backend engine (assert via command capture).
- Regression: the old unquoted writer is deleted and no test depends on its output format.

### 12.4 Golden files

A `fixtures/transfer/` directory (new) holds small golden inputs with expected import results
and expected export outputs per format/provider so that quoting and escaping behavior is
pinned by fixtures rather than by ad-hoc assertions.

## 13. Runtime verification

Recorded separately per provider and per direction.

**CSV round-trip (both providers)**

1. Export a fixture table with values containing `,`, `"`, newlines, NULLs, unicode, and a
   `bigint` beyond `2^53`; capture the file and verify it opens correctly in a third-party tool.
2. Import that file into a fresh table; capture the preview, mapping, and review step.
3. Execute the import; capture progress and the final counters.
4. Compare source and destination row counts and a checksum of ordered values; capture the
   comparison.

**Large import (both providers)**

5. Import a 100k-row CSV with `time`-measured duration and observed memory ceiling; capture
   progress screenshots and the job record.
6. Cancel at ~50%; capture the cancelled job, committed-row count, and the row count in the
   destination table, proving the reported count matches reality.

**Failure paths**

7. Import a file with a type violation and a NOT NULL violation; capture the error report with
   line numbers and the failed-row count.
8. Import with `PerBatch` and abort mid-file; capture the committed prefix warning and the
   actual row count.

**Excel/JSON**

9. Import an `.xlsx` with a date column, a formula column, and a merged header; capture the
   inferred mapping and the resulting values.
10. Import an NDJSON file with a nested object; capture the nested-value policy decision and
    the stored value.

**Export formats**

11. Export the same result set as CSV, TSV, XLSX, JSON, SQL INSERT; capture each file and the
    lossless-integer assertion.
12. Generate a `COPY` script on PostgreSQL and execute it against a scratch table; capture the
    result. (SQLite: capture that the option is absent.)

**Backup / restore**

13. Run a backup on each provider with progress captured; cancel one mid-run and verify no
    partial artifact remains.
14. Run a restore on each provider; capture the confirmation, the restore progress, and the
    post-restore re-introspection summary. Capture the SQLite "disconnect first" precondition
    UI.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | C01 | Transfer Engine (streaming pipeline + job model) | v0.3 | P1 | §4.6 meta-store migration pattern | Every import/export/backup surface depends on chunking, progress, cancellation, and job records; building it first prevents three ad-hoc implementations |
| 2 | C02 | CSV Import | v0.3 | P1 | C01 | The most requested non-query task and the simplest reader; establishes the wizard shell and the mapping UI reused by C03/C04 |
| 3 | C03 | Excel Import | v0.3 | P2 | C02 | Shares the pipeline; adds the sheet/type-inference complexity |
| 4 | C04 | JSON Import | v0.3 | P2 | C02 | Shares the pipeline; adds nesting policy |
| 5 | C05 | Export Workbench (incl. backup/restore UX + Transfers activity) | v0.3 | P1 | C01 | Removes the lossy local writer and the rail placeholder; depends on C01 for streaming and jobs, but not on C02–C04 |

Ordering rules: C02, C03, and C04 must share one pipeline — if any of them forks its own
write path, the milestone is rejected in review. C05 may run in parallel with C03/C04 once C01
lands. Backup/restore UX completion rides in C05 because it is the same job/progress
machinery, not because it is the same feature.

## 15. Definition of done

1. **Plan folder** per milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **Streaming proof**: a 100k-row (or larger) import and export complete with a recorded
   bounded-memory measurement; no code path materializes the full dataset (reviewed and
   asserted by test).
4. **The unquoted local writer is gone** (`crates/ui/src/query_view.rs:1357-1389` deleted) and
   every export goes through `ExportService` with quoting/escaping covered by golden files.
5. **Cancellation is real**: a cancelled transfer reports the exact committed/rolled-back
   state, and the destination is verified to match that report.
6. **Progress is honest**: indeterminate before the first measurement, rate-limited afterwards,
   and always terminated by a terminal event.
7. **Job records persist** across restart with counters and status; retention is bounded and
   tested; records contain no row values, credentials, or full query text.
8. **Import never interpolates values**: generated statements contain placeholders only
   (asserted by test), and `NULL`/`DEFAULT`/missing-column semantics are explicit and tested.
9. **Provider evidence recorded separately**, with SQLite-only limitations (JSON-as-TEXT,
   no `COPY`, `VACUUM INTO` backup, disconnected-target restore) stated in the UI rather than
   discovered as errors.
10. **Placeholders removed**: the Transfers activity shows real surfaces; no `COMING SOON`
    text remains for Transfer.
11. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
    `cargo test --workspace`, SQLite suite, PostgreSQL live suite including a live backup
    run).
12. **Docs updated**: capability matrix §9/§10 rows move off MISSING/BACKEND_ONLY; master goal
    §9.7 reflects reality; the resolved contradiction C11 (SQLite backup mechanism wording) and
    the `LIM-012` import limitation are updated or marked superseded.
13. **Non-goals respected**: no DB-to-DB transfer, no scheduler, no full-dataset
    materialization, no silent partial writes, no implicit table creation.

Phase C is complete when a user can import CSV/Excel/JSON into an existing table with mapping,
preview, progress, cancellation, and an error report; export any result/table/selection as
CSV/TSV/XLSX/JSON/SQL with correct escaping and lossless numbers; run and monitor backup and
restore with progress; and see every one of those operations in a persistent job list — with
PostgreSQL and SQLite evidence recorded independently.
