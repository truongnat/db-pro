# Data Safety Audit — Export/Import/Backup Integrity

> Baseline SHA: `a2cf4a3`
> Issue: #128
> Source: `crates/infrastructure/src/backup/`, `crates/core/src/domain/backup.rs`

## Operation classification

| Operation | Provider | Format | Status | Source |
|---|---|---|---|---|
| Backup | PostgreSQL | Plain SQL | Implemented, not qualified | `pg_dump.rs` |
| Backup | PostgreSQL | Custom | Implemented, not qualified | `pg_dump.rs` |
| Backup | SQLite | File copy | Implemented, not qualified | `sqlite_backup.rs` |
| Restore | PostgreSQL | Plain SQL | Implemented, not qualified | `pg_dump.rs` (psql) |
| Restore | PostgreSQL | Custom | Implemented, not qualified | `pg_dump.rs` (pg_restore) |
| Restore | SQLite | File copy | Implemented, not qualified | `sqlite_backup.rs` |
| CSV Export | Both | CSV | Implemented, not qualified | `core/Cargo.toml` csv crate |
| XLSX Export | Both | XLSX | Implemented, not qualified | `core/Cargo.toml` rust_xlsxwriter |
| Import | Both | — | Deferred | No import implementation found |

## Backup audit

### PostgreSQL (`pg_dump.rs`)

**Backup flow:**
1. Shells out to `pg_dump` binary with connection args
2. Password passed via `PGPASSWORD` environment variable (not CLI arg — correct)
3. Schema/table filters passed as `-n`/`-t` args
4. Output written to `options.output_path` via `-f` flag
5. On success, reads file metadata for size reporting

**Findings:**
- **F1 [P2]**: `pg_dump` must be on PATH — not bundled. Already documented in #134.
- **F2 [P2]**: No cancellation support — `cmd.output().await` blocks until completion. Cannot cancel a running backup.
- **F3 [P2]**: No partial-file cleanup on failure. If pg_dump fails mid-write, the partial output file remains at `output_path`. No atomic write (write to temp → rename).
- **F4 [P2]**: Schema/table names passed directly as args to pg_dump. The pg_dump binary handles its own escaping, so this is safe from SQL injection. However, schema names with special characters could cause pg_dump argument parsing issues.

**Restore flow:**
1. Plain format: shells out to `psql -f input_path`
2. Custom format: shells out to `pg_restore -d database input_path`
3. Password via `PGPASSWORD` env var

**Findings:**
- **F5 [P2]**: No pre-restore validation of the backup file. A corrupted or wrong-format file will fail at the psql/pg_restore level with an opaque error.
- **F6 [P1]**: Restore overwrites the target database without confirmation or backup-first. If the user restores to a live database, existing data is silently replaced/merged depending on the SQL content.

### SQLite (`sqlite_backup.rs`)

**Backup flow:**
1. `tokio::fs::copy(src, dst)` — direct file copy

**Findings:**
- **F7 [P1]**: No WAL checkpoint before copy. If the SQLite database is in WAL mode and has uncommitted WAL data, the backup may miss recent writes or produce an inconsistent snapshot. The correct approach is `sqlite3_backup_init/step/finish` API or at minimum `PRAGMA wal_checkpoint(TRUNCATE)` before copy.
- **F8 [P2]**: No file locking during copy. If the database is being written to during the copy, the backup may be inconsistent.

**Restore flow:**
1. `tokio::fs::copy(src, dst)` — overwrites the database file

**Findings:**
- **F9 [P1]**: Restore overwrites the target database file without backup or confirmation. If the app is connected to the database during restore, the file replacement will cause undefined behavior (open file handles to a deleted/replaced file).

## Export audit (CSV/XLSX)

**Source**: `csv` and `rust_xlsxwriter` crates in `core/Cargo.toml`.

No dedicated export module found in infrastructure. Export is likely implemented at the application/frontend layer using these crates. Without seeing the full export path, the audit is limited to the dependency evidence.

**Findings:**
- **F10 [P2]**: Export implementation not fully traced — needs frontend export module audit.
- **F11 [P2]**: No cancellation support for exports visible in the backend.

## Import audit

**Status**: Deferred/not supported in v0.1.

No import implementation found in the codebase. The `BackupEngine` trait only supports backup/restore, not data import.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 3 | F6 (restore without backup-first), F7 (SQLite WAL inconsistency), F9 (restore overwrites live DB) |
| P2 | 8 | F1-F5, F8, F10, F11 |

**Recommendation**:
- F6, F9: Add confirmation dialog before restore operations (UI-level fix).
- F7: Document SQLite backup limitation — WAL checkpoint recommended before backup.
- F1-F5, F8: ACCEPT RC1 with documentation.
