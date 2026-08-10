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
- Capture enabled state from `pg_trigger.tgenabled`
- CREATE TRIGGER with function body
- DROP TRIGGER with IF EXISTS
- ALTER TRIGGER ... DISABLE/ENABLE

**SQLite:**
- Introspect via `sqlite_master WHERE type = 'trigger'`
- Parse SQL body to extract: table, timing, event, definition
- CREATE TRIGGER with SQL body
- DROP TRIGGER
- No native disable support (capability-gated)

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
