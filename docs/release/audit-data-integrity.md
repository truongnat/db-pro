# Export / import / backup integrity and partial-failure semantics (#128)

- **Audited baseline:** `main @ a0cdebb` (issue-queue pass 3, 2026-09-15)
- **Issue:** #128 ([RC1][Data Safety] Audit export/import/backup integrity, cancellation, and
  partial-failure semantics) — parent **#14**, supports #27/#29/#95/#105
- **Rule from the issue:** inspect implementation/tests/docs only; create focused child issues for proven
  gaps. No behaviour was patched here — the three real gaps are child issue **#244**. One *documentation*
  defect (the release-facing capability matrix) was corrected in place, because the issue's own acceptance
  requires the support matrix to be exact.
- **Related, not duplicated:** #145 (SQLite backup snapshot-safety, closed except its packaged-smoke leg)
  and #91–#95 (packaged smoke) own the runtime legs; #129 owns destructive-SQL confirmation.

## 1. What v0.1 actually ships

| Operation | Provider / format | Classification | Where it lives / evidence |
|---|---|---|---|
| Export CSV / TSV from the query view | PostgreSQL + SQLite | **Implemented but not qualified** | dialog `crates/ui/src/query_view.rs:1297-1299` (CSV/TSV + free-text path); writer `export_result` `:1356-1369` → `format_result_delimited` (the shared serializer from #61). Not atomic (E-1) |
| Export JSON / XLSX | — | **Deferred / not supported** | the core `ExportService` (which owns `csv` and `rust_xlsxwriter`, `crates/core/Cargo.toml:14-15`) is **unreachable from the shipped app**: `grep -rn "export_api()" crates/native-app crates/ui crates/runtime` → 0 hits; `grep -rni "xlsx" crates/ui/src crates/native-app/src` → 0 hits. Its only callers are the legacy `crates/tauri-app` commands |
| Clipboard copy (cell / row / CSV / JSON) | both | **Release-qualified** | #61/#66 evidence: one shared escaping path, pinned by `test_copy_as_json_keeps_exact_numeric_digits`, `test_delimited_export_keeps_field_count_for_awkward_values`, `test_export_result_writes_escaped_delimited_text` |
| Import | any | **Deferred / not supported** | LIM-012 ("Import deferred"); `grep -rni "import" crates/ui/src crates/runtime/src` → no import surface |
| Backup SQLite | SQLite | **Release-qualified** (source + tests; packaged smoke owned by #91/#95) | `VACUUM INTO` into `.db-pro-<uuid>.tmp`, `flush_file` fsync, no-overwrite `hard_link` publish, sidecar (`-wal`/`-shm`/`-journal`) removal — `crates/infrastructure/src/backup/sqlite_backup.rs:131,57-76,155,19-56`; 8 regression tests (#145 evidence) |
| Backup PostgreSQL | PostgreSQL | **Implemented but not qualified** | `pg_dump` argv construction `crates/infrastructure/src/backup/pg_dump.rs:85-112`, password via `PGPASSWORD` env (never argv), destination reserved with `create_new(true)` and deleted on failure (`:54-74,133-136`). Requires `pg_dump` on `PATH` (LIM-015 — not bundled), and a running backup **cannot be cancelled** (E-3) |
| Restore SQLite | SQLite | **Release-qualified** | staged copy → `SQLITE_OPEN_READ_ONLY` + `PRAGMA quick_check` → sidecar removal → atomic rename, then the introspection cache is dropped (`sqlite_backup.rs:175+`, `backup_service.rs:89`); defect tests from #145 fail-before/pass-after |
| Restore PostgreSQL | PostgreSQL | **Implemented but not qualified** | `psql -f` (plain) / `pg_restore` (custom), `pg_dump.rs:140-170`; requires the tools on `PATH`; **no transaction boundary and no partial-failure recovery** (E-2) |

## 2. Findings

| # | Severity | Finding | Disposition |
|---|---|---|---|
| E-1 | **P2** | **Export is not atomic and has no cancellation.** `export_result` builds the whole file in memory and calls `std::fs::write(path, output)` directly (`crates/ui/src/query_view.rs:1363`): no temp-file+rename, no overwrite check, no partial-file cleanup, no progress and no cancellation. A write that fails midway (disk full, I/O error, removable volume removed) leaves a **truncated file at the destination that looks like a complete export**, and the failure is reported only as a status-bar string. Memory: the entire result is materialised as one `String` (the grid is virtualised precisely to avoid that) | **`Fix RC1` — child issue #244** |
| E-2 | **P2** | **PostgreSQL restore is not transactional.** The plain path runs `psql -f <dump>` with no `--single-transaction`, and the custom path runs `pg_restore` without `--single-transaction`/`--exit-on-error`; a mid-script failure leaves the target database **partially restored**. The app reports the subprocess error honestly but says nothing about the resulting state, and offers no rollback. By contrast the SQLite restore is validate-then-rename and therefore all-or-nothing | **`Fix RC1` — child issue #244** |
| E-3 | **P2** | **A running backup/restore cannot be cancelled.** `RuntimeCommand::CancelOperation` is declared (`crates/runtime/src/worker.rs:205`) and handled (`:1563`) but **never constructed anywhere in the workspace** — the only two occurrences are those lines. A long `pg_dump` blocks until it finishes or hits the query timeout; there is no Stop affordance for backup/restore in the UI | **`Fix RC1` — child issue #244** |
| E-4 | **P2** | **The release-facing capability matrix overstated the shipped export surface** — `docs/release/provider-capability-matrix.md:109-117` listed `XLSX export` as `SUPPORTED + NOT YET QUALIFIED` "via `rust_xlsxwriter` crate" and `CSV export` "via `csv` crate" for both providers, while neither the XLSX surface nor the `csv`-based `ExportService` is reachable from the shipped app (§1) | **`Fix RC1` — fixed in this audit** (documentation only): the section now separates the shipped CSV/TSV writer from the library-only formats and carries the classification vocabulary from §1 |
| E-5 | P3 | No progress reporting for backup/restore/export: each is one blocking call whose only output is the final status message; `BackupResult.size_bytes` is the produced file's size, which is accurate | **`Accept RC1`**, recorded |
| E-6 | P3 | External tools are required and not bundled (`pg_dump`, `psql`, `pg_restore`; LIM-015), so the PostgreSQL backup/restore legs cannot pass on a host without them — the packaged smoke must record their absence as an environment fact, not as a product failure | **`Accept RC1`** — recorded so #91–#95 interpret a missing binary correctly |
| E-7 | P3 | Import has no code path at all (LIM-012): no parser, no dry-run, no transaction boundary to audit. The scope items about duplicate/key/type errors and dangerous schema assumptions are therefore vacuous for v0.1 and must not appear as "supported" anywhere | **`Defer post-v0.1`**, recorded |

**Positively verified, no action:** staged grid mutations apply as **one parameterised transaction**
(`table_data_service.rs:213-250`, delete→update→insert order, readonly re-checked), so there is no
partial-write window in the grid path; the SQLite backup and restore are snapshot-safe, fsynced, published
without overwrite and sidecar-clean (#145 evidence); export escaping is shared with the clipboard paths and
pinned by tests (#61); backup/restore secrets never enter argv (`PGPASSWORD`/`SSHPASS` via the child
environment) and the SSH path is pinned by `tunnel.rs:199-218`.

## 3. Partial-failure semantics summary (the question the issue asks)

| Operation | Failure mid-flight | State left behind | Reported as |
|---|---|---|---|
| SQLite backup | `VACUUM INTO` or publish fails | temp file removed; destination untouched (`create_new`/`hard_link` cannot overwrite) | error |
| SQLite restore | staged copy or `quick_check` fails | destination untouched (rename never happens) | error |
| PostgreSQL backup | `pg_dump` fails | the reserved file is deleted (`pg_dump.rs:133-136`) | error |
| PostgreSQL restore | `psql`/`pg_restore` fails | **target database partially restored** (E-2) | error text, no state warning |
| CSV/TSV export | `fs::write` fails midway | **truncated file at the destination** (E-1) | error string in the status bar |
| Grid apply | any statement fails | nothing applied (single transaction) | error |

## 4. Tests

| Path | Covering test(s) |
|---|---|
| Export formatting/escaping | `test_export_result_writes_escaped_delimited_text`, `test_delimited_export_keeps_field_count_for_awkward_values`, `test_copy_as_json_keeps_exact_numeric_digits` (`crates/ui/src/app_tests.rs:2872,2900,2938`) — a real file write is asserted, but **not** the atomicity/overwrite behaviour (there is none) |
| SQLite backup/restore | 6 tests in `sqlite_backup.rs` + 2 in `backup_service.rs` (#145) incl. the two defect regressions and `backup_uses_persisted_custom_secret_reference` |
| PostgreSQL backup/restore | argv construction and the `sshpass`-style secret handling are pinned (`pg_dump.rs`, `tunnel.rs:199-218`); **no test** covers a partial restore, an external tool that is missing, or a cancellation |
| Cancel path | **no test** — the command is never constructed (E-3) |
| Import | n/a (no code) |

## 5. Not claimed

- No live PostgreSQL backup/restore run was performed by this audit (the fixture is used by other suites);
  the findings are source-level and the tool-behaviour claims are quoted from the argv/src construction.
- No packaged-runtime verification: whether `pg_dump`/`psql` are present on a user's machine is #91–#95's
  observation, and their absence is an environment fact (E-6), not a product defect.
- The classification in §1 is a source-and-tests classification; "Release-qualified" here means the code is
  atomic/snapshot-safe *and* pinned by tests, not that a package-level run has passed (that is #91–#95).
