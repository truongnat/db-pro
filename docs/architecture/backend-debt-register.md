# Backend Debt Register

> Status: **Active** (P2-01 audit, P2-15 reliability audit)
> Last updated: 2026-08-08 (post PATCH 2)

Severity levels:
- **P0**: Runtime corruption / destructive / security
- **P1**: Incorrect behavior
- **P2**: Architecture debt
- **P3**: Cleanup

---

## P0 — Critical

### P0-01: `unwrap()` on `RwLock` in connector runtime path

- **File**: `crates/infrastructure/src/connector.rs`
- **Status**: ✅ **Fixed** — all 12 `unwrap()` calls replaced with `unwrap_or_else(|e| e.into_inner())`

### P0-02: `unwrap()` on `Mutex` in CancelRegistry

- **File**: `crates/tauri-app/src/cancel.rs`
- **Status**: ✅ **Fixed** — replaced with `unwrap_or_else(|e| e.into_inner())`. Registry evolved into `ExecutionRegistry`.

### P0-03: `unwrap()` on `RwLock` in ConnectionRegistry

- **File**: `crates/core/src/application/registry.rs`
- **Status**: ✅ **Fixed** — all 6 `expect()` calls replaced with `unwrap_or_else(|e| e.into_inner())`

---

## P1 — Incorrect Behavior

### P1-01: Cross-connection commands bypass application layer

- **Files**: `crates/tauri-app/src/commands/cross_connection.rs`
- **Status**: Open — deferred (requires new application service, low runtime risk)

### P1-02: `test_ssh_tunnel` command bypasses application layer

- **Files**: `crates/tauri-app/src/commands/connection.rs`
- **Status**: Open — deferred (isolated code path, low risk)

### P1-03: Error conversion loses semantic structure

- **Status**: ✅ **Fixed** (P2-02) — full SQLSTATE-aware mapping in `from_sqlx`

### P1-04: Duplicate `parse_connection_id` across 6 command files

- **Status**: Open — documented, low-risk cleanup

### P1-05: `QueryError` and `DbError` have overlapping variants

- **Status**: ✅ **Fixed** (P2-02) — `From<QueryError> for DbError` maps to specific variants

### P1-06: SQLite introspection uses string interpolation for table names

- **File**: `crates/infrastructure/src/sqlite/introspect.rs`
- **Status**: ✅ **Fixed** (P2-05) — all PRAGMA queries now use parameterized `?` placeholders

---

## P2 — Architecture Debt

### P2-01: No database capability model

- **Status**: ✅ **Fixed** (P2-03) — `DatabaseCapabilities` in `domain/capabilities.rs`

### P2-02: No query execution lifecycle tracking

- **Status**: ✅ **Fixed** (P2-04) — `QueryExecution` + `ExecutionRegistry`

### P2-03: No connection safety policy

- **Status**: ✅ **Fixed** (P2-08) — `ConnectionSafetyPolicy` in `domain/safety.rs`

### P2-04: `IntrospectResult` drops triggers and functions in DTO

- **Status**: Open — deferred (frontend doesn't consume these yet)

### P2-05: `CellValue` and `QueryParam` are structurally identical

- **Status**: Open — deferred (cosmetic, no runtime impact)

### P2-06: No persistence versioning

- **Status**: ✅ **Fixed** (P2-10) — `schema_version` table + migration registry

---

## P3 — Cleanup

### P3-01: `DdlResultDto` reports `affected_rows` for DDL

- **Status**: Open — cosmetic

### P3-02: Duplicate DDL commands

- **Status**: Open — intentional API naming

### P3-03: SQLite validation doesn't check whitespace-only path

- **Status**: Open — edge case

---

## P2-15 Reliability Audit Summary

### Runtime unwrap/expect audit

| Location | Type | Risk | Action |
|----------|------|------|--------|
| `lib.rs:34-43` | `expect()` in bootstrap | Low (startup only) | Acceptable — app cannot function without data dir |
| `lib.rs:202` | `expect()` on Tauri runner | Low (startup only) | Acceptable — fatal if runner fails |
| `sql_policy.rs:29` | `unwrap()` after `peek()` | None (guarded) | Safe — `peek() == Some` guarantees `next() == Some` |
| `ssh/tunnel.rs:127` | `unwrap()` on `local_addr()` | Low | ✅ **Fixed** — now uses proper error propagation |

### Test-only unwrap (acceptable)

All `unwrap()` calls in `#[cfg(test)]` modules are acceptable.

### Summary

| Severity | Original | Fixed | Remaining |
|----------|----------|-------|----------|
| P0 | 3 | 3 | 0 |
| P1 | 5+1 | 4 | 2 |
| P2 | 6 | 4 | 2 |
| P3 | 3 | 0 | 3 |
