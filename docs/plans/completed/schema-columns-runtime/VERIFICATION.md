# Verification — Schema Columns Runtime

## Build

- Rust: `cargo check --workspace` — PASS
- Frontend: `npx vitest run` — 1291 tests, 103 files, PASS

## Test coverage

### column-mutation-risk.test.ts (53 tests)

- No-op detection
- Rename → medium risk + dependency warning
- Type changes: varchar→text (low), text→integer (high), integer→bigint (low), integer→text (medium), numeric→integer (high), timestamp→date (medium), date→timestamp (low)
- Nullable: →NOT NULL (medium), →nullable (low)
- Default: add (low), remove (low)
- Combined mutations: worst-risk propagation
- SQL quoting: special characters, identifier escaping
- Default formatting: quoted literals, numbers, NULL, TRUE, FALSE, CURRENT_TIMESTAMP, function calls, bare strings, injection guards
- validateDataType: simple types, parameterized, multi-word, array, rejection of semicolons/comments/keywords/non-numeric params/special chars

### Schema service tests (5 tests)

- Service layer integration

## Runtime contract verification

```
UI action (edit column dialog)
→ classifyColumnMutation() generates deterministic SQL + risk
→ useExecuteDdlBatch sends all statements atomically
→ Rust execute_ddl_batch validates each against safety policy
→ connector.execute_batch wraps in DB transaction
→ PostgreSQL: pool.begin() / SQLite: unchecked_transaction()
→ On success: COMMIT + cache invalidate (all 5 surfaces)
→ On failure: automatic ROLLBACK, error surfaced to UI
→ React Query refetches → all UI surfaces show DB truth
```

## Pending runtime verification (needs live DB)

- [ ] PostgreSQL: rename + type + nullable combined → verify all three applied
- [ ] PostgreSQL: type narrowing failure → verify rollback (no partial changes)
- [ ] SQLite: same matrix as PostgreSQL
- [ ] Cache: after mutation, DDL tab shows updated CREATE TABLE
- [ ] Cache: after rename, dependencies tab reflects new name
