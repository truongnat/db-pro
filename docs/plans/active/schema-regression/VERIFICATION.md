# S7 — Schema Regression Verification

State: REVIEW (regression matrix complete; gap-filling tests added; CI green; all Cubic reviews classified)

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | [31436500294](https://github.com/truongnat/db-pro/actions/runs/31436500294) | PASS |
| Frontend checks (tsc + eslint + prettier + build + test) | [31436500294](https://github.com/truongnat/db-pro/actions/runs/31436500294) | PASS |
| Rust checks (prior) | [31435411795](https://github.com/truongnat/db-pro/actions/runs/31435411795) | PASS |
| Frontend checks (prior) | [31435411795](https://github.com/truongnat/db-pro/actions/runs/31435411795) | PASS |
| Frontend tests (local) | npm test -- --run | 105 files, 1319 tests PASS |
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
- [x] CI passing (consecutive green runs on latest HEAD)
- [x] VERIFICATION.md updated
- [x] Cubic ci.yml P3 (apt-get update): CONFIRMED, FIXED in `314dcfe`
- [x] Cubic ci.yml P2 (--include-ignored): REJECTED — by design, PG tests must run in CI
