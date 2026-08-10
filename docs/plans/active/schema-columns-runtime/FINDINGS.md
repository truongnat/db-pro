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

---

## P1-004 — Plan status contradicts verification evidence

Status: FIXED

Evidence:
STATUS.md showed COMPLETED but VERIFICATION.md had 5 unchecked runtime items. CHECKLIST.md ticked runtime matrix without live DB evidence.

Fix:
Status changed to RUNTIME_VERIFY. Plans moved from completed/ back to active/. CHECKLIST.md runtime items unchecked.

---

## P1-005 — SQLite column ALTER capability not proven

Status: FIXED

Evidence:
Column mutation SQL generator is PostgreSQL-oriented. SQLite has limited ALTER TABLE support. The dialog did not pass driver/dialect info to the classifier.

Fix:
- `classifyColumnMutation()` accepts optional `driverType` param. When `"sqlite"`, type/nullable/default changes are skipped and listed in `unsupported[]`.
- `ColumnEditDialog` receives `driverType` prop. When SQLite: type input, nullable checkbox, and default input are disabled. Warning banner explains SQLite limitations.
- `db-object-tab-content.tsx` looks up driver from `useConnectionStore` and passes it to the dialog.
- 6 new tests in `column-mutation-risk.test.ts` cover all gating scenarios.

---

## P1-006 — Schema mutation error normalization missing

Status: FIXED

Evidence:
`column-edit-dialog.tsx` caught errors with `err instanceof Error ? err.message : String(err)`. `apiInvoke()` throws structured `TranslatedError` objects, producing `[object Object]`.

Fix:
Created `normalizeAppError()` in `frontend/src/commons/utils/normalize-app-error.ts`. Handles `TranslatedError`, native `Error`, and unknown shapes. Used in `column-edit-dialog.tsx`.

---

## P2-003 — Backend cache invalidation failure silently swallowed

Status: FIXED

Evidence:
After DDL commit, if `self.cache.invalidate()` fails, the error was logged but API returned success.

Fix:
- `SchemaService.execute_ddl` / `execute_ddl_batch` return `(u64, bool)` — bool = cache invalidation success.
- `DdlResultDto` includes `cache_invalidated: bool`.
- Frontend `useExecuteDdlBatch` logs `console.warn` when `cacheInvalidated` is false.

---

## P2-004 — No rollback regression test

Status: FIXED

Evidence:
`741a18d` changed 12 production files but added no test files.

Fix:
4 new integration tests in `crates/infrastructure/tests/integration.rs`:
- `execute_batch_all_valid_commits` — valid batch commits
- `execute_batch_middle_failure_rolls_back_all` — stmt1 rolled back when stmt2 fails
- `execute_batch_first_failure_rolls_back` — no partial execution
- `execute_batch_empty_statements` — edge case
