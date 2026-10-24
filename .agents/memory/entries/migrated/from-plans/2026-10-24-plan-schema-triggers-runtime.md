---
schema_version: 1
id: "MEM-PLAN-schema-triggers-runtime-89f63a3e7a29"
title: "Plan archive: Schema Triggers Runtime"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, schema-triggers-runtime]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "89f63a3e7a29bf9a18b1265c77e0b4efa1b522edd2bb9ed8ec22c852e1532c8c"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/schema-triggers-runtime/PLAN.md"
  - "docs/plans/active/schema-triggers-runtime/FINDINGS.md"
  - "docs/plans/active/schema-triggers-runtime/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/schema-triggers-runtime/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# S4 — Schema Triggers Runtime

## Goal

Trigger introspection, creation, drop, and DDL representation must satisfy:

```
DB trigger definition
→ accurate introspection (identity, table, timing, event, body)
→ safe CREATE/DROP with confirmation
→ DDL representation that matches DB truth
→ refresh after mutation
→ provider-aware behavior (PostgreSQL vs SQLite)
```

## Problem

Current `Trigger` struct is minimal:

```rust
pub struct Trigger {
    pub name: String,
    pub event: String,    // SQLite: full SQL body (misuse); PG: event_manipulation only
    pub action: String,   // SQLite: always empty; PG: action_statement only
}
```

Missing: table association, timing (BEFORE/AFTER), full definition, enabled state, schema qualification.

SQLite introspection reads `tbl_name` but discards it (`let _table`). PostgreSQL introspection misses `event_object_table`, `action_timing`, and trigger function body.

## Current behavior

- Triggers are introspected but not associated with their table
- No trigger CREATE/DROP UI or backend commands
- No trigger DDL representation
- No trigger tab in the frontend schema UI
- SQLite `event` field misused to store full SQL body
- PostgreSQL only captures event_manipulation and action_statement

## Target behavior

### Domain model

```rust
pub struct Trigger {
    pub name: String,
    pub table_name: String,
    pub schema: String,           // PG: trigger schema; SQLite: empty
    pub timing: String,           // BEFORE | AFTER | INSTEAD OF
    pub event: String,            // INSERT | UPDATE | DELETE | TRUNCATE (comma-separated for multi-event)
    pub definition: String,       // full trigger body/function
    pub enabled: bool,            // PG: pg_trigger.tgenabled; SQLite: always true (no disable support)
}
```

### Provider behavior

**PostgreSQL:**
- Introspect via `information_schema.triggers` + `pg_trigger` + `pg_proc`
- Capture: name, table, schema, timing, event, action_statement, trigger function body
- Map `pg_trigger.tgenabled`: `D` disabled; `O`, `R`, and `A` enabled.
- Exclude internal catalog triggers (`pg_trigger.tgisinternal`).
- CREATE TRIGGER with function body
- DROP TRIGGER with IF EXISTS
- `ALTER TABLE <qualified table> ENABLE|DISABLE TRIGGER <quoted trigger name>`

**SQLite:**
- Introspect via `sqlite_master WHERE type = 'trigger'`
- Parse SQL body to extract: table, timing, event, definition
- CREATE TRIGGER with SQL body
- DROP TRIGGER
- No native disable support (capability-gated)

**MySQL:**
- Trigger create/drop and introspection remain supported.
- Individual trigger enable/disable is unsupported; the capability gate MUST prevent SQL emission.

**SQL Server:**
- Individual trigger enable/disable remains gated unsupported by this provider.

### Frontend
- Trigger tab in schema object viewer
- List triggers for current table
- CREATE trigger dialog with SQL editor
- DROP trigger with destructive confirmation
- DDL viewer showing trigger definition

## Invariants

1. One trigger belongs to exactly one table
2. Trigger identity is `{name}` (unique within a database in SQLite, within a schema in PG)
3. Trigger DDL must be reproducible from introspected truth
4. DROP must require explicit confirmation
5. Trigger mutations invalidate introspection cache

## Scope

- Trigger introspection (PostgreSQL + SQLite)
- Trigger CREATE (PostgreSQL + SQLite)
- Trigger DROP (PostgreSQL + SQLite)
- Trigger DDL representation
- Trigger enabled/disabled (PostgreSQL only, capability-gated for SQLite)
- Frontend trigger tab
- Cache invalidation after trigger mutations
- Special identifier handling (quoted names)

## Explicit out of scope

- Trigger editing (ALTER) — future enhancement
- Trigger debugging/stepping
- System/internal triggers
- Event triggers (PostgreSQL)

## Safety requirements

- DROP TRIGGER requires destructive confirmation
- Trigger body SQL is user-provided — validate syntax but do not sanitize content
- Read-only connections must not allow CREATE/DROP
- Trigger mutations invalidate all schema caches

## Test strategy

### Rust integration tests
- SQLite: CREATE trigger → perform DML → observe effect → introspect → DROP → verify removal
- SQLite: trigger with special identifiers (quoted table/column names)
- PostgreSQL: same matrix (requires live PG)
- Rollback: trigger creation failure must not leave partial state

### Frontend tests
- Trigger list rendering from mock introspection data
- CREATE dialog validation
- DROP confirmation flow

## Runtime verification strategy

- SQLite: full automated test coverage (in-memory DB)
- PostgreSQL: source verification + live test when credentials available
- UI: manual verification with running app

## Completion criteria

- P0 = 0, P1 = 0
- Trigger introspection works for both providers (source-level for PG)
- CREATE/DROP trigger works for SQLite (automated), PG (source-level)
- DDL representation matches DB truth
- Frontend trigger tab functional
- Cache invalidation verified
- Special identifier handling tested


## Source: `FINDINGS.md`

# S4 — Schema Triggers Runtime Findings

## Confirmed Findings

### F1: Trigger domain struct was minimal (FIXED)
Original `Trigger` had only `name`, `event` (misused), `action` (always empty).
Missing: `table_name`, `schema`, `timing`, `definition`, `enabled`.
**Resolution**: Extended struct with all fields. Both providers updated.

### F2: SQLite discarded `tbl_name` (FIXED)
`introspect_triggers()` read `tbl_name` from `sqlite_master` but assigned it to `_table` (unused).
**Resolution**: Now populates `table_name` field.

### F3: SQLite `event` field misused (FIXED)
SQLite stored the full CREATE TRIGGER SQL body in the `event` field.
**Resolution**: `definition` now holds the SQL body. `event` is parsed from the header.

### F4: PostgreSQL missing table/timing/enabled (FIXED)
Query only selected `trigger_name, event_manipulation, action_statement`.
**Resolution**: Added `event_object_table`, `event_object_schema`, `action_timing`, and `pg_trigger.tgenabled` join.

### F5: Parser false positive on body DML keywords (FIXED)
Initial parser searched entire SQL for INSERT/UPDATE/DELETE. An AFTER UPDATE trigger
whose body does INSERT was incorrectly classified as INSERT.
**Resolution**: Split at first `BEGIN`, search only header portion.

### F6: SQLite INSTEAD OF only on views (TEST NOTE)
SQLite does not support `INSTEAD OF` triggers on tables — only on views.
Test adjusted to use a view for INSTEAD OF testing.

## Review Triage

### Cubic Review #1 (PR #8)

**P1: pg_trigger join keyed only on name — cross-match bug** (FIXED)
Join now constrains on schema + table via pg_class/pg_namespace, preventing
duplicate rows when the same trigger name exists on multiple tables.

**P2: PostgreSQL definition omits trigger function body** (ACCEPTED AS FOLLOW-UP)
`action_statement` stores the EXECUTE FUNCTION call, not the function body.
Retrieving the full body via `pg_get_functiondef(tgfoid)` is a DDL display
enhancement, deferred to S5 DDL scope.

### Cubic Review #2 (PR #8)

**P2 #1: Trigger row Drop button only fills form** (FIXED)
Renamed button to "Select to drop" to clarify it pre-fills the name
rather than executing the drop immediately.

**P2 #2: Parser matches BEGIN in trigger names** (FIXED)
Changed from `find("BEGIN")` to `find(" BEGIN")` (with leading space)
to avoid matching identifiers containing "begin" as substring.

**P2 #3: COUNT(*) assertion checked rows.len() not value** (FIXED)
Changed to check `CellValue::Int64(1)` from the actual count cell.


## Source: `VERIFICATION.md`

# S4 — Schema Triggers Runtime Verification

State: IMPLEMENTED IN SOURCE; focused checks and live-provider verification pending.

## Historical CI evidence (predates the native lifecycle implementation)

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + cargo test) | [31423584979](https://github.com/truongnat/db-pro/actions/runs/31423584979) | PASS |
| Frontend checks (typecheck + lint + prettier + test + build) | [31423584979](https://github.com/truongnat/db-pro/actions/runs/31423584979) | PASS |
| PR #8 mergeable | MERGEABLE | — |

## Pre-implementation audit findings (RESOLVED)

### Domain model gaps (FIXED)

Extended `Trigger` struct now includes: `table_name`, `schema`, `timing`, `definition`, `enabled`.

### SQLite introspection (FIXED)

- `table_name` populated from `sqlite_master.tbl_name`
- `parse_sqlite_trigger_sql()` extracts timing/event from header (before BEGIN)
- `definition` contains full SQL body

### PostgreSQL introspection (FIXED)

- Joins `information_schema.triggers` with `pg_trigger` for enabled state
- Captures `event_object_table`, `event_object_schema`, `action_timing`

## Parser bug fix

Initial implementation searched entire SQL body for INSERT/UPDATE/DELETE keywords,
which produced false positives when the trigger body contained DML (e.g. AFTER UPDATE
trigger whose body does INSERT). Fixed by splitting at first BEGIN and searching only
the header portion.

## Provider matrix

| Provider | Introspection | CREATE | DROP | DDL viewer | Enable/Disable |
|---|---|---|---|---|---|
| PostgreSQL | Source + unit mapping tests | existing | existing | DONE | Implemented via typed mutation; focused checks/live PG pending |
| SQLite | source + integration test | existing | existing | DONE | Unsupported; capability-gated and mutation planner emits no SQL |
| MySQL | existing | existing | existing | existing | Unsupported; capability-gated and mutation planner emits no SQL |
| SQL Server | existing | existing | existing | existing | Unsupported; no SQL builder, capability reason shown |
## DDL viewer integration

`get_table_ddl` now appends trigger definitions after the CREATE TABLE statement.
- SQLite: uses full CREATE TRIGGER SQL from `sqlite_master`
- PostgreSQL: reconstructs from timing/event/action_statement parts

- Native Explorer trigger rows display introspected Enabled/Disabled state.
- PostgreSQL exposes Enable/Disable through the context menu; other providers show a disabled reason.
- Actions create typed `ObjectMutationRequest`s and flow through Schema Workbench preview/apply and `UiCommand::ExecuteDdl`.
- Capability-gated by `CapabilityFeature::TriggerEnableDisable`.

Focused tests/checks and native UI captures are pending.

