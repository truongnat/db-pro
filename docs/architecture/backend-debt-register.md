# Backend Debt Register

> Status: **Active** (P2-01 audit)
> Last updated: 2026-08-08

Severity levels:
- **P0**: Runtime corruption / destructive / security
- **P1**: Incorrect behavior
- **P2**: Architecture debt
- **P3**: Cleanup

---

## P0 — Critical

### P0-01: `unwrap()` on `RwLock` in connector runtime path

- **File**: `crates/infrastructure/src/connector.rs` lines 74, 83, 138–140, 157–158, 165–167, 174–176, 185–187
- **Issue**: `self.handle_driver.read().unwrap()`, `self.inner_handles.write().unwrap()` etc. If the `RwLock` is ever poisoned (e.g. a panic in a prior holder), every subsequent operation panics, crashing the app.
- **Fix**: Replace `.unwrap()` with `.unwrap_or_else(|e| e.into_inner())` to recover from poison, or use `parking_lot::RwLock` which cannot be poisoned.
- **Status**: Open

### P0-02: `unwrap()` on `Mutex` in CancelRegistry

- **File**: `crates/tauri-app/src/cancel.rs` lines 21, 27, 37
- **Issue**: Same poison-panic risk as P0-01. CancelRegistry is on the query execution hot path.
- **Fix**: Use `parking_lot::Mutex` or `unwrap_or_else(|e| e.into_inner())`.
- **Status**: Open

### P0-03: `unwrap()` on `RwLock` in ConnectionRegistry

- **File**: `crates/core/src/application/registry.rs` lines 24, 31, 40, 45, 52
- **Issue**: Same poison-panic risk. ConnectionRegistry is used by every query/connection operation.
- **Fix**: Use `parking_lot::RwLock` or recover from poison.
- **Status**: Open

---

## P1 — Incorrect Behavior

### P1-01: Cross-connection commands bypass application layer

- **Files**: `crates/tauri-app/src/commands/cross_connection.rs` lines 46–91
- **Issue**: `get_object_dependencies`, `list_partitions`, `list_tablespaces`, `rename_schema_object` call `PostgresConnector` directly from the Tauri command, bypassing any application service. This violates the layer architecture and means no error mapping, no safety policy, no audit trail.
- **Fix**: Route through an application service (e.g. extend `SchemaService` or create a dedicated service).
- **Status**: Open

### P1-02: `test_ssh_tunnel` command bypasses application layer

- **File**: `crates/tauri-app/src/commands/connection.rs` lines 119–132
- **Issue**: Calls `CompositeConnector::test_ssh_tunnel` directly instead of through `ConnectionService`.
- **Fix**: Add `ConnectionService::test_ssh_tunnel` and route through it.
- **Status**: Open

### P1-03: Error conversion loses semantic structure

- **File**: `crates/infrastructure/src/error.rs`
- **Issue**: `from_sqlx` maps all database errors to either `AuthFailed`, `QueryFailed`, or `Internal`. The original SQLSTATE code is discarded. Frontend cannot distinguish constraint violations, permission errors, syntax errors, etc.
- **Fix**: Extend `DbError` with richer variants that carry SQLSTATE codes and structured details.
- **Status**: Open (addressed in P2-02)

### P1-04: Duplicate `parse_connection_id` across 5 command files

- **Files**: `query.rs:186`, `schema.rs:103`, `table_data.rs:107`, `export.rs:39`, `cross_connection.rs:94`, `user_management.rs:9`
- **Issue**: Same function copy-pasted 6 times. If the error format changes, all copies must be updated.
- **Fix**: Extract to a shared helper in `dto.rs` or a `commands::util` module.
- **Status**: Open

### P1-05: `QueryError` and `DbError` have overlapping variants

- **Files**: `crates/core/src/domain/query.rs:103-131`, `crates/core/src/domain/error.rs:33-67`
- **Issue**: Both enums define timeout, connection, validation, and internal variants. The `From<QueryError> for DbError` conversion collapses distinct error types (e.g. `PermissionDenied` → `QueryFailed`).
- **Fix**: Unify into a single error taxonomy (P2-02).
- **Status**: Open

---

## P2 — Architecture Debt

### P2-01: No database capability model

- **Issue**: `CompositeConnector` uses `match driver { DriverType::Postgres => ..., DriverType::SQLite => ... }` throughout (lines 126–154, 180–183, 193–196, 202–205, etc.). Adding a new driver requires modifying every match arm.
- **Fix**: Implement `DatabaseCapabilities` registry (P2-03).
- **Status**: Open

### P2-02: No query execution lifecycle tracking

- **Issue**: Queries are fire-and-forget. No `QueryExecutionId`, no status tracking, no deterministic cleanup. Cancel works via a side-channel (`CancelRegistry`) disconnected from the query itself.
- **Fix**: Formalize query lifecycle (P2-04).
- **Status**: Open

### P2-03: No connection safety policy

- **Issue**: No backend enforcement of read-only connections. A "read-only" connection can still execute `DROP TABLE` if the frontend sends it.
- **Fix**: Implement `ConnectionSafetyPolicy` (P2-08).
- **Status**: Open

### P2-04: `IntrospectResult` drops triggers and functions in DTO

- **File**: `crates/tauri-app/src/dto.rs:316-338`
- **Issue**: `IntrospectResultDto` does not include `triggers` or `functions` fields, even though the domain `IntrospectResult` has them. Data is silently lost in the DTO conversion.
- **Fix**: Add `triggers` and `functions` to `IntrospectResultDto`.
- **Status**: Open

### P2-05: `CellValue` and `QueryParam` are structurally identical

- **File**: `crates/core/src/domain/query.rs:3-47`
- **Issue**: `QueryParam` and `CellValue` have identical variants. They should share a common type or one should be an alias.
- **Fix**: Consider unifying or using a type alias.
- **Status**: Open

### P2-06: No persistence versioning

- **Issue**: `SQLiteMetaStore` creates tables without schema version tracking. Future migrations will have no foundation.
- **Fix**: Add version table and migration registry (P2-10).
- **Status**: Open

---

## P3 — Cleanup

### P3-01: `DdlResultDto` reports `affected_rows` for DDL

- **File**: `crates/tauri-app/src/dto.rs:502-506`
- **Issue**: DDL operations (CREATE INDEX, DROP TRIGGER) don't meaningfully return "affected rows". The field is always 0.
- **Fix**: Use a success indicator or remove the field.
- **Status**: Open

### P3-02: `create_index` / `drop_index` / `create_trigger` / `drop_trigger` all call the same `execute_ddl`

- **File**: `crates/tauri-app/src/commands/schema.rs:52-94`
- **Issue**: These commands are identical to `execute_ddl`. They exist as separate commands only for frontend naming clarity, but duplicate the same backend path.
- **Fix**: Keep as-is for API clarity, but document the aliasing.
- **Status**: Open

### P3-03: `ConnectionConfig` validation doesn't check SQLite database path existence

- **File**: `crates/core/src/domain/connection.rs:137`
- **Issue**: SQLite driver branch does nothing in `validate()`. A completely empty database path is caught, but a path with only whitespace is not.
- **Fix**: Add `database.trim().is_empty()` check for SQLite.
- **Status**: Open

---

## Summary

| Severity | Count | Action |
|----------|-------|--------|
| P0 | 3 | Fix immediately (poison-panic recovery) |
| P1 | 5 | Fix in P2-02 / P2-03 / this patch |
| P2 | 6 | Address across P2 milestones |
| P3 | 3 | Low priority cleanup |
