---
schema_version: 1
id: "MEM-PLAN-schema-relations-runtime-54346592b23d"
title: "Plan archive: Schema Relations Runtime"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, schema-relations-runtime]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "54346592b23d1db1851f2215fb200a870500ab355abda3f1f4ee076375f93c48"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/schema-relations-runtime/PLAN.md"
  - "docs/plans/active/schema-relations-runtime/FINDINGS.md"
  - "docs/plans/active/schema-relations-runtime/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/schema-relations-runtime/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# S3 — Schema Relations Runtime Verification

State: RUNTIME_VERIFY
Branch: main (merged via PR #7, squash commit 7facb95)

## Goal

Foreign-key relations must preserve constraint identity, ordered column mappings and database truth across introspection, DDL reconstruction and the relation UI.

```text
database FK constraint
→ provider introspection
→ relation rows with stable constraint identity
→ table info / reconstructed DDL
→ Foreign Keys UI
→ target-table navigation
```

## Baseline evidence

The existing model stores one `ForeignKey` row per column mapping. This is compatible with single-column relations but exposed a composite-FK correctness gap:

- PostgreSQL introspection returns multiple rows with the same real constraint name for a composite FK.
- SQLite previously synthesized the name from `from_column`, so each column mapping of one composite FK received a different identity.
- `build_create_table_ddl()` emitted one `ADD CONSTRAINT` statement per mapping; PostgreSQL composite FKs therefore reconstructed as duplicate constraint names instead of one multi-column constraint.
- `ForeignKeyList` rendered one row per mapping and keyed rows by `fk.name`, producing duplicate keys for PostgreSQL composite FKs.

## Scope

- preserve one stable constraint identity across all column mappings
- preserve composite mapping order
- reconstruct one multi-column FK constraint in generated DDL
- render one relation row in the Foreign Keys UI
- keep target-table navigation
- prove SQLite FK enforcement on a live in-memory provider
- independently account for PostgreSQL runtime evidence

## Non-goals

- create/drop FK editor UI
- automatic SQLite table rebuild for adding a relation
- ER diagram composite-edge presentation; tracked under S6
- ON UPDATE / ON DELETE action editing
- deferrable constraint editing

## Implementation

### SQLite introspection

Use `PRAGMA foreign_key_list` `id` as the shared synthetic constraint identity and preserve the provider's `seq` row order. Composite mappings now share `table_fk_<id>`.

### DDL reconstruction

Group introspected FK rows by constraint identity + target table before generating DDL. Emit one statement:

```sql
ALTER TABLE ...
ADD CONSTRAINT ...
FOREIGN KEY (col_a, col_b)
REFERENCES ... (target_a, target_b);
```

### Frontend relation view

Normalize scalar mapping rows with `groupForeignKeys()` and render joined ordered columns as one relation row.

## Provider matrix

| Provider | Introspection source | Automated evidence | Live/runtime evidence |
|---|---|---|---|
| PostgreSQL | real constraint name + `src.ord` ordering | source/unit coverage | NOT VERIFIED |
| SQLite | PRAGMA `id` + `seq` | test added, execution pending | NOT VERIFIED until the command executes |

## Acceptance criteria

- [x] composite SQLite mappings share one stable constraint identity in source
- [x] DDL reconstruction groups composite mappings into one constraint in source
- [x] Foreign Keys UI groups composite mappings into one relation row in source
- [x] regression tests added for grouping and DDL reconstruction
- [x] SQLite runtime test added for composite introspection + FK enforcement
- [x] Rust/frontend quality gates executed — integrated CI PASS at 7facb95
- [ ] PostgreSQL composite FK runtime evidence recorded
- [ ] UI relation list/navigation runtime evidence recorded
- [ ] P0 = 0
- [ ] P1 = 0

S3 stays in `RUNTIME_VERIFY` until PG live + UI runtime evidence is recorded.


## Source: `FINDINGS.md`

# S3 — Schema Relations Runtime Findings

## P1-001 — Composite FK DDL reconstruction emitted duplicate constraints

Status: FIXED IN SOURCE; CI PASS (tests executed and passed in CI run 31412787979).

Evidence:
- provider introspection represents a composite FK as multiple column-mapping rows,
- PostgreSQL rows share the real constraint name,
- `build_create_table_ddl()` previously emitted one `ADD CONSTRAINT` per row.

Failure scenario:
A composite FK `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)` was reconstructed as two separate constraints with the same constraint name. Executing that DDL would be invalid and would not preserve the original relation semantics.

Fix:
Group mapping rows by constraint identity and target before emitting DDL; join ordered source/target columns into one `FOREIGN KEY (...) REFERENCES ... (...)` clause.

## P1-002 — SQLite composite FK mappings had different synthetic identities

Status: FIXED IN SOURCE; CI PASS (tests executed and passed in CI run 31412787979).

Evidence:
SQLite `PRAGMA foreign_key_list` returns `id` and `seq`, where all mappings for one composite constraint share `id`. Previous code named each mapping from `from_column`, splitting one constraint into multiple apparent relations.

Fix:
Use `{table_name}_fk_{id}` as the shared synthetic identity (e.g., `child_fk_0` for the first FK on the `child` table). Preserve PRAGMA row order for the mappings.

## P2-001 — Foreign Keys UI rendered mapping rows rather than constraints

Status: FIXED IN SOURCE; CI PASS (frontend tests executed and passed in CI).

Evidence:
`ForeignKeyList` mapped directly over scalar FK rows and used `fk.name` as the React key. PostgreSQL composite FK rows therefore produced duplicate keys and duplicate visual relation rows.

Fix:
`groupForeignKeys()` groups mappings by relation identity. The UI renders ordered source/target column lists once per constraint.

## P2-002 — ER diagram still treats scalar FK mappings as separate edges

Status: DEFERRED TO S6 ER DIAGRAM.

The current S3 scope fixes relation truth in introspection, DDL reconstruction and the Foreign Keys table. ER visualization owns a separate presentation model and will be normalized during S6 rather than expanding S3 scope.

## Runtime gaps

- PostgreSQL composite FK provider verification: NOT VERIFIED (requires live PG credentials)
- SQLite runtime test: CI PASS
- UI relation-list/navigation runtime verification: NOT VERIFIED (requires running app)

## Gate

All source-level P1 fixes validated by CI. S3 is ready for RUNTIME_VERIFY once PostgreSQL credentials and UI runtime evidence are available.

## VPS Kilo Review #1 Triage (commit b7e9c8d)

### P1-003 — FK enforcement test did not enable PRAGMA foreign_keys
- **Classification**: CONFIRMED
- **Severity**: P1 (test proving nothing without FK enforcement enabled)
- **Fix**: Added `PRAGMA foreign_keys = ON` before table creation + readback assertion to verify pragma took effect
- **Commit**: f2a2805

### P2 — S1 atomicity test only covers SQLite
- **Classification**: CONFIRMED, ACCEPTED as known gap
- **Action**: PostgreSQL rollback coverage belongs to S1 RUNTIME_VERIFY, not S3 scope

### P2 — Frontend groupForeignKeys test coverage gaps
- **Classification**: CONFIRMED, ACCEPTED as known gap
- **Action**: Single-column FK grouping is trivially correct; no regression risk

### P2 — S3 plan has pending runtime evidence
- **Classification**: CONFIRMED, TRACKED
- **Action**: Documented in VERIFICATION.md

### Rejected patterns (correct as-is)
- SQLite FK synthetic name `{table}_fk_{id}`: correct, uses PRAGMA `id`
- `group_foreign_keys_for_ddl` omits `from_table`: safe, TableInfo already filtered
- `\^@` separator in frontend key: safe, SQL identifiers cannot contain null bytes
- DDL `ALTER TABLE ADD CONSTRAINT` on SQLite: viewer-only, not executed

## Cubic Review Triage

### P3 — FINDINGS.md doc accuracy (P1-002 format)
- **Classification**: CONFIRMED, FIXED in d1d27d7

### P3 — Duplicated test setup helper
- **Classification**: CONFIRMED, DEFERRED (polish, not merge blocker)

### P3 — VPS review prompt `${BASE_REF}` placeholder
- **Classification**: CONFIRMED, FIXED in ea3312f

## Cubic Review #3 Triage (commits f2a2805–ffd2c00)

### P2 — VERIFICATION.md internal contradiction (CI table vs completion decision)
- **Classification**: CONFIRMED
- **Fix**: Updated CI evidence table to reflect actual review state (Cubic COMMENTED, VPS IN_PROGRESS), updated commit ref to ffd2c00

### P3 — `cargo check` claim inaccurate
- **Classification**: CONFIRMED
- **Fix**: Replaced `cargo check → CI PASS (via cargo build)` with `cargo build → CI PASS`. Added note that `cargo check` was not run as a separate command.


## Source: `VERIFICATION.md`

# S3 — Schema Relations Runtime Verification

State: RUNTIME_VERIFY

## CI evidence

### Integrated main — `7facb95` (PR #7 squash merged)

| Gate | Result | Evidence |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | SUCCESS | CI on main@7facb95 |
| Frontend checks (typecheck + lint + format + build + test) | SUCCESS | CI on main@7facb95 |
| VPS Kilo Review | SUCCESS | PR #7 review |
| Cubic AI Review | NEUTRAL | 2 findings (P2 + P3), triaged |
| Socket Security | SUCCESS | — |

### PR #7 branch — pre-merge CI

| Gate | Result | Run |
|---|---|---|
| Rust checks | SUCCESS | [31416039171](https://github.com/truongnat/db-pro/actions/runs/31416039171) |
| Frontend checks | SUCCESS | [31416039171](https://github.com/truongnat/db-pro/actions/runs/31416039171) |
| VPS Kilo Review | SUCCESS | [31416039236](https://github.com/truongnat/db-pro/actions/runs/31416039236) |

All Rust and Frontend CI gates pass. Tests were actually executed by CI.

## Rebase notes

PR #7 was rebased onto main after PR #6 + PR #8 were already merged. One conflict resolved in `schema_service.rs`:
- Import: kept both `ForeignKey` (PR #7) + `Trigger` (PR #8)
- Tests: kept both trigger DDL tests (PR #8) + composite FK grouping test (PR #7)

## Tests added

### Rust

- `crates/infrastructure/tests/schema_relations_runtime_verification.rs`
  - creates a composite parent key
  - creates a composite child FK
  - introspects mapping identity/order
  - inserts a valid relation row
  - verifies an invalid relation row is rejected by SQLite FK enforcement

- `crates/core/src/application/schema_service.rs`
  - unit test verifies two mapping rows reconstruct as one ordered composite FK constraint

- `crates/infrastructure/tests/schema_columns_atomicity_regression.rs`
  - regression test for column atomicity

### Frontend

- `frontend/src/modules/schema/__tests__/foreign-key-groups.test.ts`
  - composite grouping/order
  - distinct constraint identity
  - cross-schema separation

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | SOURCE VERIFIED | NOT VERIFIED | source query preserves real name and ordinality; live PG execution proof still required |
| SQLite | CI PASS | NOT VERIFIED | runtime integration test executed by CI and passed |

## UI lifecycle

```text
DB foreign key
→ introspection
→ tableInfo
→ groupForeignKeys
→ Foreign Keys tab
→ target-table navigation
```

Status: CI PASS for automated gates. Live UI runtime evidence still NOT VERIFIED.

## Completion decision

S3 RUNTIME_VERIFY: all CI gates pass on integrated main. P1 fixes applied (VPS P1-003 FK pragma). Cubic review triage recorded in FINDINGS.md. Remaining:
- PostgreSQL live runtime verification (requires PG credentials)
- UI runtime verification (requires running app)

