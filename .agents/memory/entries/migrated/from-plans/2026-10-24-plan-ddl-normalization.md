---
schema_version: 1
id: "MEM-PLAN-ddl-normalization-72a79d527860"
title: "Plan archive: Ddl Normalization"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, ddl-normalization]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "72a79d527860b8fc32ea59299be9800f68f442af8be7ed1f98ee89cda31ce9d6"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ddl-normalization/PLAN.md"
  - "docs/plans/active/ddl-normalization/FINDINGS.md"
  - "docs/plans/active/ddl-normalization/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ddl-normalization/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# S5 — DDL Normalization Plan

## Goal
Normalize DDL generation, viewing, and editing across all schema object types (tables, views, indexes, foreign keys, triggers) with provider-aware syntax.

## Problem
- View DDL is not available (get_table_ddl only handles tables, not views)
- Backend DDL builder uses hardcoded quote_identifier instead of dialect-aware quoting
- DDL editor missing trigger operations
- No comprehensive DDL test coverage

## Current Behavior
- `get_table_ddl()` generates CREATE TABLE + indexes + FKs + triggers for tables
- Selecting a view and clicking DDL tab calls `get_table_ddl()` which fails (views not in tables list)
- Frontend DDL builder (`ddl-builder.ts`) is dialect-aware, but backend is not
- DDL editor supports: createTable, addColumn, dropColumn, renameTable, dropTable, createView, dropView, createIndex, dropIndex

## Target Behavior
- View DDL shows the CREATE VIEW statement from introspection
- Backend DDL uses dialect-aware quoting where possible
- DDL editor includes trigger enable/disable operations
- All DDL paths have test coverage

## Scope
- Add `get_view_ddl` method to SchemaService
- Add view DDL Tauri command or extend `get_table_ddl` to handle views
- Add dialect parameter to `build_create_table_ddl` (or accept current quoting as compatible)
- Add trigger operations to DDL editor
- Add DDL tests

## Out of Scope
- DDL round-trip fidelity (introspection may not preserve exact original formatting)
- DDL migration between providers
- DDL diff/comparison

## Safety Requirements
- DDL viewer is read-only (no execution risk)
- DDL editor already enforces safety policy (readonly connections, multi-statement rejection)
- All raw SQL fragments must pass validate_raw_fragment

## Test Strategy
- Unit tests for view DDL generation
- Unit tests for dialect-aware quoting
- Frontend tests for DDL editor trigger operations

## Completion Criteria
- View DDL displays correctly for both PostgreSQL and SQLite
- All DDL paths have test coverage
- DDL editor supports trigger operations
- CI passing (Rust + Frontend)


## Source: `FINDINGS.md`

# S5 — DDL Normalization Findings

## DDL Architecture Audit

### Backend DDL Generation
- `build_create_table_ddl()` in `schema_service.rs` generates CREATE TABLE + indexes + FKs
- `format_trigger_ddl()` appends trigger definitions (added in S4)
- `quote_identifier()` uses double-quote wrapping (compatible with both PostgreSQL and SQLite)
- `get_table_ddl()` now handles views (falls back to view definition)

### Frontend DDL Builder
- `ddl-builder.ts` is dialect-aware (uses SqlDialect for quoting)
- Supports: createTable, addColumn, dropColumn, renameTable, dropTable, createView, dropView, createIndex, dropIndex, createTrigger, dropTrigger, setTriggerEnabled
- `ddl-capabilities.ts` tracks per-driver feature support

### DDL Viewer
- Monaco-based read-only editor
- Copy to clipboard functionality
- Open in query tab option

### DDL Editor
- Visual form for DDL generation
- Capability-gated operations (shows warning for unsupported ops)
- Live preview with syntax highlighting

## Confirmed Findings

### F1: View DDL was missing (FIXED)
`get_table_ddl()` only searched `introspect.tables`, causing view DDL requests to fail with "table not found". Fixed by checking `introspect.views` first and returning the view definition.

### F2: Backend quoting is compatible but not dialect-aware (DEFERRED)
`quote_identifier()` in schema_service.rs uses double-quote wrapping which works for both PostgreSQL and SQLite. While not using the dialect system, this is functionally correct. Full dialect integration would require threading the connection's dialect through the service layer.

### F3: DDL editor missing trigger toggle operations (FIXED)
The DDL editor's `DdlOperation` type now includes `enableTrigger`/`disableTrigger`. Trigger name input field added to DDL editor form. Capability-gated for PostgreSQL only.

## Cubic Review Findings (PR #8)

### CR1: PostgreSQL trigger function body omitted (P2 — FIXED)
`action_statement` from `information_schema.triggers` only returns `EXECUTE FUNCTION func_name()`, not the function body. Fixed by:
- Adding `function_def` field to `Trigger` domain struct
- Extending PG introspection SQL to `LEFT JOIN pg_proc` and fetch `pg_get_functiondef(pg_proc.oid)`
- Updating `format_trigger_ddl` to emit function definition BEFORE CREATE TRIGGER statement (correct replay order)
- Propagating `functionDef` through `TriggerDto` (Rust) and `TriggerDto` (TypeScript)

### CR2: SQLite BEGIN parsing fragile for trigger names containing BEGIN (P2 — FIXED)
`parse_sqlite_trigger_sql` used `upper.find(" BEGIN")` which could match inside identifiers like `trg_BEGIN_audit`. Fixed by:
- Extracting `find_trigger_header()` helper that walks forward past double-quoted identifiers
- Counting unbalanced quotes to detect when a match is inside a quoted identifier
- Added 5 unit tests covering standard, BEFORE, INSTEAD OF, unquoted BEGIN-in-name, and quoted BEGIN-in-name cases

### CR3: PG DDL ordering — CREATE FUNCTION must precede CREATE TRIGGER (P1 — FIXED)
`format_trigger_ddl` emitted CREATE TRIGGER before the function definition, causing replay failures. Fixed by emitting `function_def` first, then the CREATE TRIGGER statement.

### CR4: SQLite trigger schema mismatch (P1 — FIXED)
SQLite trigger introspection set `schema: String::new()` (empty), but `get_table_ddl` filters by `tr.schema == schema` where schema is `"main"`. Fixed by setting `schema: "main".into()` in SQLite trigger introspection.

### CR5: SQLite trigger toggle emits unsupported ALTER TABLE syntax (P2 — FIXED)
`buildSetTriggerEnabled` emitted `ALTER TABLE … ENABLE/DISABLE TRIGGER` for all dialects. SQLite doesn't support this. Fixed by returning empty string when `dialect.driver === "sqlite"`.


## Source: `VERIFICATION.md`

# S5 — DDL Normalization Verification

State: REVIEW (All Cubic review findings resolved; CI passing)

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + clippy + build + test) | [31429564670](https://github.com/truongnat/db-pro/actions/runs/31429564670) | PASS |
| Frontend checks (tsc + eslint + prettier + build + test) | [31429564670](https://github.com/truongnat/db-pro/actions/runs/31429564670) | PASS |
| Frontend typecheck (local) | tsc --noEmit | 0 errors |
| Frontend tests (local) | vitest run | 105 files, 1318 tests PASS |
| PR #8 mergeable | MERGEABLE | — |

## Changes

### View DDL (FIXED)
- `get_table_ddl` now checks `introspect.views` before failing
- Returns view definition directly from introspection data
- Unit test added: `get_table_ddl_view`

### DDL Editor Trigger Operations (DONE)
- `enableTrigger`/`disableTrigger` added to `DdlOperation` type
- Trigger name input field in DDL editor form
- Trigger operations in DDL type selector
- Capability check: SQLite shows unsupported warning
- Translation keys (en + ja)
- 6 new tests (3 builder + 3 capability + 3 preview)

### Cubic P2 Fixes

#### CR1: PostgreSQL trigger function body (FIXED)
- Added `function_def` field to `Trigger` domain struct (with `#[serde(default)]`)
- PG introspection SQL extended: `LEFT JOIN pg_proc ON pg_proc.oid = pg_t.tgfoid` + `pg_get_functiondef(pg_proc.oid)`
- `format_trigger_ddl` appends function definition after CREATE TRIGGER when present
- `TriggerDto` (Rust) + `TriggerDto` (TypeScript) updated
- New test: `format_trigger_ddl_includes_function_def_when_present`

#### CR2: SQLite BEGIN parsing (FIXED)
- Extracted `find_trigger_header()` helper with quote-aware search
- Walks forward past double-quoted identifiers to avoid matching BEGIN inside names
- 5 new unit tests: standard, BEFORE, INSTEAD OF, unquoted BEGIN-in-name, quoted BEGIN-in-name

#### CR3: PG DDL ordering (FIXED)
- `format_trigger_ddl` now emits `function_def` BEFORE `CREATE TRIGGER` statement
- Ensures DDL replay succeeds (function exists before trigger references it)
- Added ordering assertion in test

#### CR4: SQLite trigger schema (FIXED)
- Changed `schema: String::new()` → `schema: "main".into()` in SQLite trigger introspection
- `get_table_ddl` filter now correctly includes SQLite triggers

#### CR5: SQLite trigger toggle (FIXED)
- `buildSetTriggerEnabled` returns empty string for SQLite dialect
- `ALTER TABLE ENABLE/DISABLE TRIGGER` is PostgreSQL-only

### Provider Matrix

| Provider | View DDL | Trigger Toggle | DDL Editor | Function Def |
|---|---|---|---|---|
| PostgreSQL | DONE | DONE (ALTER TABLE ENABLE/DISABLE TRIGGER) | DONE (with preview) | DONE (pg_get_functiondef) |
| SQLite | DONE | N/A (capability-gated) | DONE (with warning) | N/A (full SQL from sqlite_master) |

