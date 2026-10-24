---
schema_version: 1
id: "MEM-PLAN-schema-regression-d0da53671524"
title: "Plan archive: Schema Regression"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, schema-regression]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d0da53671524833c3d6840a89e5e50babdbc00dd0417f16ccb4671bd1f3bd761"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/schema-regression/PLAN.md"
  - "docs/plans/active/schema-regression/FINDINGS.md"
  - "docs/plans/active/schema-regression/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/schema-regression/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# S7 — Full Schema Regression

## Goal

S7 is the closure program for S1–S6. It is NOT another UI feature.

Build a deterministic regression matrix covering:

- S1: Columns
- S2: Indexes
- S3: Relations / Foreign Keys
- S4: Triggers
- S5: DDL
- S6: ER Diagram

For each feature, evaluate both providers (PostgreSQL, SQLite) across:

- CREATE
- READ / INTROSPECT
- UPDATE / ALTER
- DROP
- REFRESH
- ERROR
- ROLLBACK
- READ-ONLY SAFETY
- SPECIAL IDENTIFIERS
- CACHE CONSISTENCY
- UI CONSISTENCY

## Problem

S1–S6 have been implemented and individually verified, but no single document
provides a holistic view of what is proven vs. what remains pending across the
entire schema surface. S7 closes this gap.

## Target behavior

A deterministic regression matrix (FINDINGS.md) that, for each S1–S6 feature ×
operation × provider cell, states one of:

- PASS — automated test or CI evidence proves this cell
- SOURCE — verified at source level but no runtime test
- NOT VERIFIED — not yet proven
- NOT_APPLICABLE — operation does not apply to this feature/provider

Plus a release-style schema readiness report.

## Scope

- Audit existing test coverage for S1–S6
- Identify gaps and add targeted tests where feasible
- Build the regression matrix
- Produce the schema readiness report
- Any accepted P0/P1 reopens the affected feature

## Explicit out of scope

- New UI features
- Rewriting S1–S6 implementations
- Manual runtime testing (documented as NOT VERIFIED)

## Invariants

1. Every cell in the matrix must have evidence or be explicitly NOT VERIFIED
2. PostgreSQL and SQLite are verified independently
3. Tests must prove invariants, not merely exercise code paths
4. Any P0/P1 found during S7 reopens the affected feature

## Test strategy

- Rust: targeted integration tests for uncovered cells
- Frontend: verify existing unit test coverage for DDL/ER cells
- CI: PG service container + SQLite in-memory for automated evidence
- UI: manual — documented as NOT VERIFIED

## Completion criteria

- P0 = 0, P1 = 0
- Regression matrix complete with evidence for every cell
- Schema readiness report produced
- All feasible automated gaps filled with tests


## Source: `FINDINGS.md`

# S7 — Full Schema Regression Matrix

## Evidence key

- **PASS** — automated test or CI evidence proves this cell
- **SOURCE** — verified at source level (code review, type system) but no dedicated runtime test
- **NOT VERIFIED** — not yet proven
- **N/A** — operation does not apply to this feature/provider

---

## S1 — Columns

| Operation | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| CREATE (add column) | PASS | SOURCE | DDL builder: `buildAddColumn` tested; backend: `execute_ddl` |
| READ / INTROSPECT | PASS | PASS | `integration.rs::introspect_columns_include_nullable`; `pg_integration.rs::pg_introspect_tables` |
| ALTER (rename) | PASS | SOURCE | Frontend: `column-mutation-risk.test.ts` — rename classified medium risk, SQL contains `RENAME COLUMN` |
| ALTER (type change) | PASS | SOURCE | Frontend: `column-mutation-risk.test.ts` — varchar→text low risk, narrowing high risk |
| ALTER (nullable) | PASS | SOURCE | Frontend: `column-mutation-risk.test.ts` — SET NOT NULL / DROP NOT NULL tested |
| ALTER (default) | PASS | SOURCE | Frontend: `column-mutation-risk.test.ts` — SET DEFAULT / DROP DEFAULT tested |
| DROP (remove column) | PASS | SOURCE | DDL builder: `buildDropColumn` tested |
| Combined mutation | PASS | SOURCE | Frontend: `column-mutation-risk.test.ts` — multi-operation batch tested |
| ERROR | PASS | SOURCE | `query_with_invalid_sql` proves error path; `reject_multi_statement` prevents batch injection |
| ROLLBACK (batch atomicity) | PASS | SOURCE | SQLite: `handle_execute_batch` uses `unchecked_transaction()` + commit; PG: `execute_batch` wraps in transaction |
| READ-ONLY safety | SOURCE | SOURCE | `safety_policy_for` enforces readonly; `validate_against_policy` rejects DDL |
| Special identifiers | PASS | SOURCE | `query_unicode_table`, `query_weird_name_table`; `quote_identifier` used in DDL builder |
| Cache consistency | SOURCE | SOURCE | `execute_ddl` calls `cache.invalidate()` after every mutation |
| UI consistency | NOT VERIFIED | NOT VERIFIED | Requires manual UI verification |

---

## S2 — Indexes

| Operation | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| CREATE (normal) | PASS | SOURCE | `schema_indexes_runtime_verification.rs::verify_create_and_drop_index` |
| CREATE (unique) | PASS | SOURCE | Same test — verifies `email_idx.unique == true` |
| CREATE (composite) | PASS | SOURCE | Same test — verifies `composite_idx.columns == ["email", "name"]` |
| CREATE (column order) | PASS | SOURCE | Same test — composite index columns verified in order |
| READ / INTROSPECT | PASS | PASS | `integration.rs::introspect_indexes`; `pg_integration.rs::pg_introspect_indexes` |
| DROP | PASS | SOURCE | `schema_indexes_runtime_verification.rs` — DROP INDEX + verify gone |
| REFRESH | SOURCE | SOURCE | Introspection re-fetches after cache invalidation |
| ERROR | PASS | SOURCE | Invalid SQL rejected; `execute_ddl` validates safety policy |
| READ-ONLY safety | SOURCE | SOURCE | DDL on readonly connection rejected by policy |
| Special identifiers | SOURCE | SOURCE | `buildCreateIndex` uses `quote_identifier`; no dedicated test with quoted names |
| Cache consistency | SOURCE | SOURCE | `create_index` / `drop_index` commands invalidate cache |
| UI consistency | NOT VERIFIED | NOT VERIFIED | Requires manual UI verification |

---

## S3 — Relations / Foreign Keys

| Operation | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| READ / INTROSPECT (single FK) | PASS | PASS | `integration.rs::introspect_foreign_keys`; `pg_integration.rs::pg_introspect_foreign_keys` |
| READ / INTROSPECT (composite FK) | PASS | PASS | `integration.rs::introspect_primary_keys` (composite PK); FK columns verified individually |
| Target schema/table identity | PASS | PASS | `to_table` + `to_schema` populated for both providers |
| Column mapping order | PASS | PASS | `from_column` / `to_column` per FK entry; composite FK preserves order |
| Self-referencing FK | PASS | SOURCE | `employees.manager_id → employees.id` in fixture; introspected correctly |
| Cross-schema FK | N/A | SOURCE | PostgreSQL supports cross-schema FK; SQLite single-database only |
| DDL reconstruction | SOURCE | SOURCE | `get_table_ddl` includes FK constraints in CREATE TABLE |
| ER compatibility | PASS | PASS | `edge-builder.test.ts` — 9 tests covering composite grouping, cross-schema, self-ref |
| REFRESH | SOURCE | SOURCE | Cache invalidation after schema mutation |
| Special identifiers | SOURCE | SOURCE | Quoted FK table/column names handled by `quote_identifier` |
| UI consistency | NOT VERIFIED | NOT VERIFIED | Requires manual UI verification |

---

## S4 — Triggers

| Operation | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| CREATE | PASS | SOURCE | `schema_triggers_runtime_verification.rs::trigger_lifecycle_create_introspect_drop` |
| READ / INTROSPECT (name) | PASS | PASS | `integration.rs::introspect_triggers`; `pg_integration.rs::pg_introspect_triggers` |
| READ / INTROSPECT (timing) | PASS | PASS | SQLite: `timing == "AFTER"`; PG: `timing == "BEFORE"` |
| READ / INTROSPECT (event) | PASS | PASS | SQLite: `event == "UPDATE"`; PG: `event == "UPDATE"` |
| READ / INTROSPECT (definition) | PASS | PASS | SQLite: full SQL from `sqlite_master`; PG: reconstructed from parts |
| READ / INTROSPECT (function_def) | N/A | PASS | `pg_integration.rs` — `function_def.contains("update_timestamp")` |
| READ / INTROSPECT (enabled) | PASS | PASS | SQLite: always `true`; PG: from `pg_trigger.tgenabled` |
| READ / INTROSPECT (schema) | PASS | PASS | SQLite: `"main"` (CR4 fix); PG: `"public"` |
| DROP | PASS | SOURCE | `schema_triggers_runtime_verification.rs` — DROP TRIGGER + verify gone |
| DML → observe trigger effect | PASS | SOURCE | `trigger_lifecycle` — UPDATE fires trigger → audit_log row created |
| BEFORE trigger | PASS | SOURCE | `trigger_before_insert_introspection` — RAISE(ABORT) tested |
| INSTEAD OF (views) | PASS | N/A | `trigger_instead_of_on_view` — SQLite-only feature |
| Multiple triggers same table | PASS | SOURCE | `multiple_triggers_on_same_table` — 2 triggers, distinct timing/event |
| Special identifiers | PASS | SOURCE | `trigger_with_special_identifiers` — unicode table name |
| Enable/Disable | N/A | SOURCE | SQLite: no native disable (capability-gated); PG: ALTER TABLE ENABLE/DISABLE TRIGGER |
| DDL representation | PASS | PASS | `format_trigger_ddl` handles both paths; CR3 fix: function_def before CREATE TRIGGER |
| BEGIN parsing safety | PASS | N/A | `find_trigger_header` — quote-aware, 5 unit tests (CR2 fix) |
| REFRESH | SOURCE | SOURCE | Cache invalidation after trigger mutation |
| READ-ONLY safety | SOURCE | SOURCE | DDL on readonly connection rejected by policy |
| UI consistency | NOT VERIFIED | NOT VERIFIED | Requires manual UI verification |

---

## S5 — DDL

| Operation | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| Table DDL (CREATE TABLE) | PASS | SOURCE | `build_create_table_ddl` in schema_service.rs |
| Table DDL includes triggers | PASS | PASS | `get_table_ddl` appends triggers; CR3 fix: function_def before CREATE TRIGGER |
| View DDL | PASS | PASS | `get_table_ddl` falls back to view definition; verified for both providers |
| Index DDL | SOURCE | SOURCE | `buildCreateIndex` / `buildDropIndex` in ddl-builder.ts |
| Trigger DDL (CREATE) | PASS | SOURCE | `buildCreateTrigger` — 2 tests in ddl-builder-comprehensive |
| Trigger DDL (DROP) | PASS | SOURCE | `buildDropTrigger` — 1 test |
| Trigger toggle (enable/disable) | N/A | SOURCE | `buildSetTriggerEnabled` — SQLite returns "" (CR5 fix); PG generates ALTER TABLE |
| Quoted identifiers | PASS | SOURCE | `quote_identifier` used throughout; tested with unicode/weird names |
| Defaults in DDL | PASS | SOURCE | Column defaults included in CREATE TABLE reconstruction |
| Constraints in DDL | PASS | SOURCE | PK, FK, NOT NULL, UNIQUE in CREATE TABLE |
| Multi-statement policy | SOURCE | SOURCE | `reject_multi_statement` prevents injection via semicolons |
| DDL editor preview | PASS | PASS | `ddl-builder-comprehensive.test.ts` — preview rendering for all operation types |
| Capability flags | PASS | PASS | `ddl-capabilities-comprehensive.test.ts` — 5 trigger-toggle tests |
| Cache invalidation | SOURCE | SOURCE | `execute_ddl` invalidates cache after every mutation |
| UI consistency | NOT VERIFIED | NOT VERIFIED | Requires manual UI verification |

---

## S6 — ER Diagram

| Operation | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| Node identity (schema.table) | PASS | PASS | `edge-builder.test.ts` — stable keys test |
| Composite FK → single edge | PASS | PASS | `edge-builder.test.ts` — composite FK merging test |
| Same-name FK different tables | PASS | PASS | `edge-builder.test.ts` — key includes `fromTable` (Cubic CR1 fix) |
| Self-referencing edges | PASS | PASS | `edge-builder.test.ts` — self-referencing test |
| Cross-schema visibility | PASS | PASS | `edge-builder.test.ts` — cross-schema exclusion test (Cubic CR2 fix) |
| Hidden table exclusion | PASS | PASS | `edge-builder.test.ts` — fromTable not visible → no edge |
| Empty FK list | PASS | PASS | `edge-builder.test.ts` — empty list → empty result |
| Edge ID stability | PASS | PASS | `er-diagram.tsx` uses `group.key` for edge ID |
| Layout algorithm | PASS | PASS | `layout.test.ts` — layout tests |
| Column click navigation | SOURCE | SOURCE | `table-node.tsx` dispatches custom event; `er-diagram.tsx` handles navigation |
| Position persistence | SOURCE | SOURCE | localStorage-based manual position persistence |
| Provider consistency | PASS | PASS | Edge builder is provider-agnostic; uses introspection data uniformly |
| UI consistency | NOT VERIFIED | NOT VERIFIED | Requires manual UI verification |

---

## Cross-cutting concerns

| Concern | SQLite | PostgreSQL | Evidence |
|---|---|---|---|
| Safety policy (readonly) | SOURCE | SOURCE | `safety_policy_for` + `validate_against_policy` |
| Multi-statement rejection | SOURCE | SOURCE | `reject_multi_statement` in `sql_policy.rs` |
| Identifier quoting | PASS | SOURCE | `quote_identifier` used in all DDL generation; tested with unicode/weird names |
| Cache invalidation on mutation | SOURCE | SOURCE | `execute_ddl` / `execute_ddl_batch` / `create_index` / `drop_index` / `create_trigger` / `drop_trigger` all invalidate |
| Error normalization | PASS | SOURCE | `DbError` enum with typed variants; `from_rusqlite` conversion |
| Connection safety | SOURCE | SOURCE | `ConnectionSafetyPolicy` — readonly vs full_access |
| Transaction atomicity (batch) | PASS | SOURCE | SQLite: `unchecked_transaction()` + commit; PG: `execute_batch` in transaction |

---

## Schema Readiness Report

### Feature readiness grades

| Feature | Grade | Rationale |
|---|---|---|
| S1 Columns | **B+** | Strong frontend test coverage for all mutation types + risk classification. SQLite runtime proven. PG at source level. Rollback proven at implementation level. UI verification pending. |
| S2 Indexes | **B** | Full lifecycle proven for SQLite (create unique, create composite, drop, introspect). PG introspection proven via CI. PG lifecycle at source level. Special identifier test gap. |
| S3 Relations | **B+** | FK introspection proven for both providers. Composite FK semantics verified. Edge builder has 9 tests. DDL reconstruction at source level. UI verification pending. |
| S4 Triggers | **A-** | Most thoroughly tested feature. Full lifecycle (CREATE→DML→observe→introspect→DROP→verify) for SQLite. PG introspection via CI including function_def. 5 parser safety tests. Enable/disable capability-gated. |
| S5 DDL | **B+** | Table DDL with triggers for both providers. View DDL for both. Trigger DDL builder tested. Capability flags tested. CR1-CR5 all fixed. Multi-statement policy at source level. |
| S6 ER Diagram | **B** | Edge builder well-tested (9 tests). Cubic CR1/CR2 fixed. Layout tests pass. Navigation and persistence at source level. No provider-specific divergence. |

### Overall assessment

**Ready fo

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# S7 — Schema Regression Verification

State: REVIEW (regression matrix complete; gap-filling tests added; CI green; all Cubic reviews classified)

> **Correction (2026-09-14) — V01-06 evidence audit.** The `PASS` claims recorded below in this
> document are **not** supported by retrievable evidence and must not be read as
> verified. Audited verdict for this workstream: **`EVIDENCE_GAP`**. The audit
> (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`, §5) records what is
> missing: the live PostgreSQL half rests on ignored tests; no native-UI schema traversal artifact exists; CHECK/Unique constraint inspection has no dedicated test or evidence and its disposition is still recorded as unresolved in the risk register. The original record is retained below as history, unretracted.

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | [31444927188](https://github.com/truongnat/db-pro/actions/runs/31444927188) | PASS |
| Frontend checks (tsc + eslint + prettier + build + test) | [31444927188](https://github.com/truongnat/db-pro/actions/runs/31444927188) | PASS |
| Rust checks (prior) | [31436500294](https://github.com/truongnat/db-pro/actions/runs/31436500294) | PASS |
| Frontend checks (prior) | [31436500294](https://github.com/truongnat/db-pro/actions/runs/31436500294) | PASS |
| Frontend tests (local) | npm test -- --run | 105 files, 1319 tests PASS **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The React frontend suite was retired/archived on 2026-09-11 and no longer gates release; this count is historical only and no longer describes any current gate. |
| Rust unit tests (local) | cargo test --workspace | 232 passed, 9 ignored (PG), 0 failed **[CORRECTED 2026-09-14 — V01-06 evidence audit]** Measured today at workspace level: **811 passed / 0 failed / 19 ignored**; the `232`/`9` figures were stale. |
| PR #8 mergeable | MERGEABLE | — |

## Test inventory

### Rust integration tests (SQLite)

| File | Tests | Coverage |
|---|---|---|
| `integration.rs` | 24 | Connection, query, introspection (tables, columns, PKs, indexes, FKs, views, triggers), mutation, explain |
| `schema_indexes_runtime_verification.rs` | 1 | Index lifecycle: create unique → create composite → introspect → drop → verify |
| `schema_triggers_runtime_verification.rs` | 5 | Trigger lifecycle, BEFORE INSERT, special identifiers, INSTEAD OF, multiple triggers |

### Rust integration tests (PostgreSQL — Live Container & CI)

| File | Tests | Coverage | Status |
|---|---|---|---|
| `pg_integration.rs` | 18 | Tables, triggers (incl. function_def), views, indexes, FKs, query, composite FK detail, index lifecycle, special identifiers, temporal/network types, transaction rollback, timeout rollback | ~~PASS (18/18)~~ **[CORRECTED 2026-09-14 — V01-06 evidence audit]** `EVIDENCE_GAP`: all 18 tests are `#[ignore]`d and run as **0 passed / 18 ignored** unless `DATABASE_URL` points at an isolated PostgreSQL fixture. |
| `ssh_backup_runtime_verification.rs` | 1 | PG backup and restore through live SSH tunnel | ~~PASS (1/1)~~ **[CORRECTED 2026-09-14 — V01-06 evidence audit]** `EVIDENCE_GAP`: this test is `#[ignore]`d and runs as **0 passed / 1 ignored** unless the `DB_PRO_SSH_*` fixtures are provided. |


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
| 17 | CR6 | P3 | docs/plans/project/07-current-status.md | FIXED | P2.11 row corrected to "1319 FE + 39 Rust" |
| 18 | CR6 | P3 | VERIFICATION.md | FIXED | integration.rs count corrected 18 → 24 |
| 19 | CR7 | P2 | 0.1.0-readiness.md | FIXED | S4 line corrected — enable/disable not exercised in live PG |
| 20 | CR7 | P3 | VERIFICATION.md | FIXED | "3 consecutive green runs" → "consecutive green runs on latest HEAD" |

