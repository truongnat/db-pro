# S7 — Schema Regression Verification

State: REVIEW (regression matrix complete; gap-filling tests added; CI green; all Cubic reviews classified)

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | [31444927188](https://github.com/truongnat/db-pro/actions/runs/31444927188) | PASS |
| Frontend checks (tsc + eslint + prettier + build + test) | [31444927188](https://github.com/truongnat/db-pro/actions/runs/31444927188) | PASS |
| Rust checks (prior) | [31436500294](https://github.com/truongnat/db-pro/actions/runs/31436500294) | PASS |
| Frontend checks (prior) | [31436500294](https://github.com/truongnat/db-pro/actions/runs/31436500294) | PASS |
| Frontend tests (local) | npm test -- --run | 105 files, 1319 tests PASS |
| Rust unit tests (local) | cargo test --workspace | 232 passed, 9 ignored (PG), 0 failed |
| PR #8 mergeable | MERGEABLE | — |

## Test inventory

### Rust integration tests (SQLite)

| File | Tests | Coverage |
|---|---|---|
| `integration.rs` | 24 | Connection, query, introspection (tables, columns, PKs, indexes, FKs, views, triggers), mutation, explain |
| `schema_indexes_runtime_verification.rs` | 1 | Index lifecycle: create unique → create composite → introspect → drop → verify |
| `schema_triggers_runtime_verification.rs` | 5 | Trigger lifecycle, BEFORE INSERT, special identifiers, INSTEAD OF, multiple triggers |

### Rust integration tests (PostgreSQL — CI service container)

| File | Tests | Coverage |
|---|---|---|
| `pg_integration.rs` | 9 | Tables, triggers (incl. function_def), views, indexes, FKs, query, composite FK detail, index lifecycle, special identifiers |

### Frontend tests

| Module | Files | Tests | Coverage |
|---|---|---|---|
| schema | 8 | ~400+ | Column mutation risk, DDL builder (all operations), DDL capabilities, schema service |
| er-diagram | 2 | ~15 | Edge builder (9 tests), layout |
| data-grid | multiple | ~300+ | Grid operations |
| query | multiple | ~50+ | SQL templates, generators, explain |
| connections | multiple | ~100+ | Connection management |
| commons | multiple | ~200+ | Shared utilities |

## Gap-filling tests

| Gap | Test | Status |
|---|---|---|
| PG composite FK detail | `pg_integration.rs::pg_composite_fk_detail` | DONE — CI PASS |
| PG index lifecycle | `pg_integration.rs::pg_index_lifecycle` | DONE — CI PASS |
| PG special identifiers | `pg_integration.rs::pg_special_identifiers` | DONE — CI PASS |

## Review gate

- [x] P0 = 0
- [x] P1 = 0
- [x] P2 = 0 (all confirmed findings fixed or classified)
- [x] CI passing (consecutive green runs on latest HEAD)
- [x] VERIFICATION.md updated

## Cubic review classification (all 7 runs, 20 findings)

| # | Run | Severity | File | Classification | Evidence |
|---|---|---|---|---|---|
| 1 | CR1 | P1 | sqlite/introspect.rs | FIXED | `parse_sqlite_trigger_sql` now isolates header via `find_trigger_header()` before parsing event |
| 2 | CR1 | P1 | postgres/introspect.rs | FIXED | pg_trigger join includes `tgname + nspname + relname` (table-unique) |
| 3 | CR1 | P2 | postgres/introspect.rs | FIXED | `pg_get_functiondef` via `pg_proc` join; `format_trigger_ddl` emits function first |
| 4 | CR2 | P2 | trigger-manager.tsx | REJECTED | Button label is "Select for Drop" (two-step safety pattern), not immediate drop |
| 5 | CR2 | P2 | sqlite/introspect.rs | FIXED | `find_trigger_header()` skips quoted identifiers containing BEGIN |
| 6 | CR2 | P2 | schema_triggers_runtime_verification.rs | REJECTED | `rows.len()==1` verifies query success; `rows[0].0[0]==Int64(1)` verifies actual count |
| 7 | CR3 | P1 | ddl-builder.ts | FIXED | `buildSetTriggerEnabled` returns `""` for SQLite dialect |
| 8 | CR3 | P1 | postgres/introspect.rs | FIXED | `format_trigger_ddl` emits function_def BEFORE CREATE TRIGGER (test asserts ordering) |
| 9 | CR3 | P1 | schema_service.rs | FIXED | SQLite triggers now have `schema: "main"` matching table schema filter |
| 10 | CR3 | P3 | CHECKLIST.md | STALE | Documentation count mismatch — low priority, does not affect code |
| 11 | CR3 | P3 | ddl-normalization/VERIFICATION.md | STALE | Documentation count mismatch — low priority, does not affect code |
| 12 | CR4 | P1 | er-diagram.tsx | FIXED | `groupForeignKeys` uses `${schema}.${fromTable}.${name}` key — composite grouping + cross-table uniqueness |
| 13 | CR4 | P2 | edge-builder.ts | FIXED | Both `fromKey` and `toKey` checked against `visibleTables` |
| 14 | CR5 | P2 | ci.yml | REJECTED | `--include-ignored` is by design — PG tests must run in CI with service container |
| 15 | CR5 | P3 | ci.yml | FIXED | `sudo apt-get update` added before `apt-get install` in `314dcfe` |
| 16 | CR6 | P2 | pg_integration.rs | FIXED | Pre-cleanup `DROP INDEX IF EXISTS` added in `f4ff701` |
| 17 | CR6 | P3 | plans/07-current-status.md | FIXED | P2.11 row corrected to "1319 FE + 39 Rust" |
| 18 | CR6 | P3 | VERIFICATION.md | FIXED | integration.rs count corrected 18 → 24 |
| 19 | CR7 | P2 | 0.1.0-readiness.md | FIXED | S4 line corrected — enable/disable not exercised in live PG |
| 20 | CR7 | P3 | VERIFICATION.md | FIXED | "3 consecutive green runs" → "consecutive green runs on latest HEAD" |
