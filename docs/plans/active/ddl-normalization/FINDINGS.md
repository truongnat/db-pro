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
- Updating `format_trigger_ddl` to append function definition after CREATE TRIGGER statement
- Propagating `functionDef` through `TriggerDto` (Rust) and `TriggerDto` (TypeScript)

### CR2: SQLite BEGIN parsing fragile for trigger names containing BEGIN (P2 — FIXED)
`parse_sqlite_trigger_sql` used `upper.find(" BEGIN")` which could match inside identifiers like `trg_BEGIN_audit`. Fixed by:
- Extracting `find_trigger_header()` helper that walks forward past double-quoted identifiers
- Counting unbalanced quotes to detect when a match is inside a quoted identifier
- Added 5 unit tests covering standard, BEFORE, INSTEAD OF, unquoted BEGIN-in-name, and quoted BEGIN-in-name cases
