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

## S1.5 Runtime matrix — SOURCE ONLY, NOT RUNTIME VERIFIED

- [x] PostgreSQL transaction support via sqlx (source)
- [x] SQLite transaction support via rusqlite (source)
- [ ] PostgreSQL: rename + type + nullable combined → verify all three applied
- [ ] PostgreSQL: type narrowing failure → verify rollback (no partial changes)
- [ ] SQLite: rename column
- [x] SQLite: change type — capability-gated (disabled in UI, not sent to backend)
- [x] SQLite: nullable ON/OFF — capability-gated (disabled in UI, not sent to backend)
- [x] SQLite: default value — capability-gated (disabled in UI, not sent to backend)
- [ ] SQLite: combined mutation
- [x] Capability-gate unsupported SQLite operations in UI

## S1.6 UX

- [x] Compact diff-oriented dialog
- [x] Before → After display for all fields
- [x] Space-in-identifier warning
- [x] Risk badge + change count inline
- [x] Cmd/Ctrl+Enter keyboard shortcut preserved

## S1.7 Closure (open)

- [x] Normalize schema mutation errors (P1-3)
- [x] Add atomic rollback regression tests (P2-2)
- [x] Surface cache invalidation failure in response (P2-1)
- [x] SQLite capability-gate column operations (P1-2)
- [ ] Live PostgreSQL runtime matrix
- [ ] Verify DDL/tableInfo/dependencies refresh after mutation
- [x] Update VERIFICATION.md with evidence
- [ ] Only then: active/ → completed/, STATUS → COMPLETED
