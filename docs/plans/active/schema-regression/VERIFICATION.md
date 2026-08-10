# S7 — Schema Regression Verification

State: IMPLEMENTING (regression matrix complete; gap-filling tests added; awaiting CI)

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | [31432622639](https://github.com/truongnat/db-pro/actions/runs/31432622639) | PASS |
| Frontend checks (tsc + eslint + prettier + build + test) | [31432622639](https://github.com/truongnat/db-pro/actions/runs/31432622639) | PASS |
| Frontend tests (local) | npm test -- --run | 105 files, 1319 tests PASS |
| PR #8 mergeable | MERGEABLE | — |

## Test inventory

### Rust integration tests (SQLite)

| File | Tests | Coverage |
|---|---|---|
| `integration.rs` | 18 | Connection, query, introspection (tables, columns, PKs, indexes, FKs, views, triggers), mutation, explain |
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
| PG composite FK detail | `pg_integration.rs::pg_composite_fk_detail` | DONE |
| PG index lifecycle | `pg_integration.rs::pg_index_lifecycle` | DONE |
| PG special identifiers | `pg_integration.rs::pg_special_identifiers` | DONE |

## Review gate

- [x] P0 = 0
- [x] P1 = 0
- [ ] CI passing after gap-filling tests
- [x] VERIFICATION.md updated
