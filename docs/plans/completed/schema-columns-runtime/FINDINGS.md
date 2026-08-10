# Findings — Schema Columns Runtime

## P1-001 — Multi-statement schema mutation is non-atomic

Status: FIXED
Commit: 741a18d

Evidence:
`column-edit-dialog.tsx` executed `classified.sql` sequentially via `for..of executeDdl.mutateAsync(stmt)`. If statement N failed, statements 1..N-1 were already committed.

Fix:
New `execute_batch` on `DbConnector` trait wraps all statements in a single DB transaction. PostgreSQL uses `pool.begin()`, SQLite uses `unchecked_transaction()`. Frontend calls `executeBatch.mutateAsync(classified.sql)` once.

---

## P1-002 — Schema cache invalidation incomplete

Status: FIXED
Commit: 741a18d

Evidence:
`useExecuteDdl` only invalidated `schema-introspect` and `schemaCatalogStore`. After column edit, `schema-table-info`, `schema-table-ddl`, and `object-dependencies` could serve stale data.

Fix:
New `invalidateAllSchemaCaches()` helper invalidates all five surfaces: introspect, tableInfo, tableDdl, dependencies, and Zustand catalog store.

---

## P1-003 — Screenshot/source mismatch (build identity)

Status: CLOSED (not a bug)

Evidence:
Screenshot showed only "Allow NULL" + "Low Risk" for a rename+nullable change. Source at fb91c4e correctly classifies rename as medium risk.

Conclusion:
App was running an old build, not fb91c4e. No source fix needed.

---

## P2-001 — Column name with spaces lacks warning

Status: FIXED
Commit: 741a18d

Fix:
Dialog shows "Contains spaces — will require quoting in SQL" when new name differs and contains spaces.

---

## P2-002 — Dialog too spacious for IDE density

Status: FIXED
Commit: 741a18d

Fix:
Reshaped to compact diff-oriented layout: `sm:max-w-md`, `h-7` inputs, `text-xs`/`text-[11px]`/`text-[10px]` typography, Before → After diff summary, inline risk badge + change count.
