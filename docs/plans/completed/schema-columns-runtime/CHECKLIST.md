## S1.1 Runtime identity

- [x] Source `classifyColumnMutation()` verified correct at fb91c4e
- [x] Rename → medium risk, nullable → low risk, worst-risk propagation works
- [x] Screenshot mismatch attributed to old build, not source bug

## S1.2 Preview

- [x] Rename preview generates `RENAME COLUMN` SQL
- [x] Nullable toggle generates `DROP NOT NULL` / `SET NOT NULL`
- [x] Rename + nullable combined → both operations listed, risk = medium
- [x] Type + default combined → correct SQL for each
- [x] Worst-risk propagation across all operation types

## S1.3 Atomic execution

- [x] `execute_batch` added to `DbConnector` trait
- [x] PostgreSQL: `pool.begin()` transaction with commit/rollback
- [x] SQLite: `unchecked_transaction()` with commit/rollback
- [x] CompositeConnector delegates to inner
- [x] `execute_ddl_batch` Tauri command registered
- [x] Frontend uses single `executeBatch.mutateAsync(classified.sql)` call
- [x] Each statement validated against safety policy before execution

## S1.4 Cache

- [x] `schema-introspect` invalidated
- [x] `schema-table-info` invalidated
- [x] `schema-table-ddl` invalidated
- [x] `object-dependencies` invalidated
- [x] `schemaCatalogStore` (Zustand) invalidated
- [x] Backend `IntrospectionCache` invalidated

## S1.5 Runtime matrix

- [x] PostgreSQL transaction support via sqlx
- [x] SQLite transaction support via rusqlite
- [x] Success path: all statements commit
- [x] Failure path: transaction rolls back automatically
- [x] Partial failure impossible (single transaction)

## S1.6 UX

- [x] Compact diff-oriented dialog
- [x] Before → After display for all fields
- [x] Space-in-identifier warning
- [x] Risk badge + change count inline
- [x] Cmd/Ctrl+Enter keyboard shortcut preserved
