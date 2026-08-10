# S4 — Schema Triggers Runtime Findings

## Pre-implementation audit

### Finding 1 — Trigger domain struct missing critical fields

Status: CONFIRMED

The `Trigger` struct only has `name`, `event`, `action`. Missing `table_name`, `timing`, `definition`, `enabled`, `schema`. This means triggers cannot be properly associated with their table, and the DDL reconstruction cannot produce accurate output.

### Finding 2 — SQLite introspection discards table name

Status: CONFIRMED

`introspect_triggers()` in `sqlite/introspect.rs` reads `tbl_name` from `sqlite_master` but assigns it to `_table` (unused). The trigger's table association is lost.

### Finding 3 — SQLite event field misused

Status: CONFIRMED

The `event` field stores the full SQL body for SQLite triggers, while `action` is always empty. This is semantically incorrect — `event` should be INSERT/UPDATE/DELETE, and the full SQL should be in `definition`.

### Finding 4 — PostgreSQL introspection incomplete

Status: CONFIRMED

PostgreSQL introspection only captures `trigger_name`, `event_manipulation`, and `action_statement`. Missing: `event_object_table`, `action_timing`, `action_condition`, trigger function body (requires `pg_proc` join), and enabled state (requires `pg_trigger.tgenabled`).

### Finding 5 — No trigger CREATE/DROP commands

Status: CONFIRMED

No backend commands exist for creating or dropping triggers. No Tauri commands, no SchemaService methods, no frontend UI.

### Finding 6 — No trigger DDL reconstruction

Status: CONFIRMED

`build_create_table_ddl()` does not include triggers in its output. Trigger DDL is not reconstructed anywhere.
