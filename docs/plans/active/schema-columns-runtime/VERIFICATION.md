# Verification — Schema Columns Runtime

## Build

- Rust: `cargo check --workspace` — PASS
- Frontend: `npx tsc --noEmit` — PASS
- Frontend: `npx vitest run` — all tests PASS

## Test coverage

### column-mutation-risk.test.ts (59 tests)

- No-op detection
- Rename → medium risk + dependency warning
- Type changes: varchar→text (low), text→integer (high), integer→bigint (low), integer→text (medium), numeric→integer (high), timestamp→date (medium), date→timestamp (low)
- Nullable: →NOT NULL (medium), →nullable (low)
- Default: add (low), remove (low)
- Combined mutations: worst-risk propagation
- SQL quoting: special characters, identifier escaping
- Default formatting: quoted literals, numbers, NULL, TRUE, FALSE, CURRENT_TIMESTAMP, function calls, bare strings, injection guards
- validateDataType: simple types, parameterized, multi-word, array, rejection of semicolons/comments/keywords/non-numeric params/special chars
- **SQLite capability gating** (6 tests): rename allowed, type/nullable/default blocked → `unsupported[]`, combined draft allows rename + blocks 3 others, postgres default has no unsupported

### Schema service tests (5 tests)

- Service layer integration

### Integration tests — atomic batch rollback (4 tests)

- `execute_batch_all_valid_commits` — valid batch commits, all effects visible
- `execute_batch_middle_failure_rolls_back_all` — stmt1 valid, stmt2 invalid → stmt1 rolled back
- `execute_batch_first_failure_rolls_back` — no partial execution on first-statement failure
- `execute_batch_empty_statements` — empty array returns 0 affected

## S1.7 Runtime closure evidence

### P1-3: Error normalization

- **File**: `frontend/src/commons/utils/normalize-app-error.ts`
- **Evidence**: `normalizeAppError()` handles `TranslatedError` (structured), native `Error`, and unknown shapes. Used in `column-edit-dialog.tsx` instead of `err instanceof Error ? err.message : String(err)`. No more `[object Object]`.

### P2-1: Cache invalidation failure surfaced

- **Backend**: `SchemaService.execute_ddl` / `execute_ddl_batch` return `(u64, bool)` — bool = cache invalidation success
- **DTO**: `DdlResultDto` includes `cache_invalidated: bool`
- **Frontend**: `useExecuteDdlBatch` logs `console.warn` when `cacheInvalidated` is false
- **Files**: `schema_service.rs`, `dto.rs`, `commands/schema.rs`, `schema.queries.ts`, `schema.service.ts`

### P2-2: Rollback regression tests

- **File**: `crates/infrastructure/tests/integration.rs` — 4 new tests
- **Evidence**: Tests prove atomic behavior: when any statement fails, all prior statements are rolled back. SQLite uses `unchecked_transaction()`, PostgreSQL uses `pool.begin()`.

### P1-2: SQLite capability gating

- **Classifier**: `classifyColumnMutation()` accepts optional `driverType` param. When `"sqlite"`, type/nullable/default changes are skipped and listed in `unsupported[]`.
- **UI**: `ColumnEditDialog` receives `driverType` prop. When SQLite: type input, nullable checkbox, and default input are `disabled`. Warning banner explains SQLite's limited ALTER TABLE support.
- **Wiring**: `db-object-tab-content.tsx` looks up driver from `useConnectionStore` and passes it to the dialog.
- **Tests**: 6 new tests in `column-mutation-risk.test.ts` covering all gating scenarios.
- **Files**: `column-mutation-risk.ts`, `column-edit-dialog.tsx`, `db-object-tab-content.tsx`, `column-mutation-risk.test.ts`

## Runtime contract verification

```
UI action (edit column dialog)
→ classifyColumnMutation(draft, schema, table, driverType) generates deterministic SQL + risk
→ SQLite: unsupported ops disabled in UI, not sent to backend
→ useExecuteDdlBatch sends all statements atomically
→ Rust execute_ddl_batch validates each against safety policy
→ connector.execute_batch wraps in DB transaction
→ PostgreSQL: pool.begin() / SQLite: unchecked_transaction()
→ On success: COMMIT + cache invalidate (all 5 surfaces)
→ On failure: automatic ROLLBACK, error normalized via normalizeAppError()
→ cacheInvalidated flag surfaced in response
→ React Query refetches → all UI surfaces show DB truth
```

## Pending runtime verification (needs live DB)

- [ ] PostgreSQL: rename + type + nullable combined → verify all three applied
- [ ] PostgreSQL: type narrowing failure → verify rollback (no partial changes)
- [ ] SQLite: rename column via live dialog
- [ ] Cache: after mutation, DDL tab shows updated CREATE TABLE
- [ ] Cache: after rename, dependencies tab reflects new name
