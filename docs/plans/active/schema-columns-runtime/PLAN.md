# Schema Columns Runtime Verification

Baseline: fb91c4e
Branch: main (pre-workflow)
Commit: 741a18d

## Goal

Column mutation must satisfy:

```
UI draft
→ deterministic SQL preview
→ risk classification
→ atomic execution
→ DB truth
→ fresh introspection
→ all UI surfaces synchronized
```

## Scope

- rename
- data type change (widening/narrowing)
- nullable toggle
- default value (literal/expression)
- combined mutations (2–4 changes at once)
- rollback on failure
- cache invalidation (introspect + tableInfo + tableDdl + dependencies + catalog)
- runtime UX (compact diff-oriented dialog)

## Out of scope

- indexes
- relations
- triggers
- ER diagram

## Implementation summary

### P1-2 Preview correctness
Source `classifyColumnMutation()` already correct at fb91c4e — rename processed before nullable, worst-risk propagation works. Screenshot was from old build.

### P1-3 Atomic schema mutation
- Added `execute_batch()` to `DbConnector` trait
- PostgreSQL: `pool.begin()` → loop `sqlx::query().execute(&mut *tx)` → `tx.commit()`
- SQLite: `unchecked_transaction()` → `execute_batch()` per statement → `commit()`
- New `execute_ddl_batch` Tauri command
- Frontend `useExecuteDdlBatch` replaces sequential `for..of executeDdl.mutateAsync(stmt)` loop

### P1-4 Cache invalidation
`invalidateAllSchemaCaches()` now covers:
- `schema-introspect`
- `schema-table-info`
- `schema-table-ddl`
- `object-dependencies`
- `schemaCatalogStore` (Zustand)

### P2-1 Space warning
Dialog warns when column name contains spaces.

### P2-2 Dialog polish
Compact diff-oriented layout with Before → After display.

## Files changed

```
crates/core/src/ports/db_connector.rs
crates/core/src/application/schema_service.rs
crates/infrastructure/src/connector.rs
crates/infrastructure/src/postgres/connector.rs
crates/infrastructure/src/sqlite/actor.rs
crates/infrastructure/src/sqlite/connector.rs
crates/tauri-app/src/commands/schema.rs
crates/tauri-app/src/lib.rs
frontend/src/commons/di/registry.ts
frontend/src/modules/schema/components/column-edit-dialog.tsx
frontend/src/modules/schema/queries/schema.queries.ts
frontend/src/modules/schema/services/schema.service.ts
```

## Test results

- Rust: all workspace tests pass
- Frontend: 1291 tests pass (103 files)
