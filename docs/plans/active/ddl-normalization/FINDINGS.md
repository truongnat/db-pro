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

### F3: DDL editor missing trigger toggle operations (TODO)
The DDL editor's `DdlOperation` type doesn't include trigger enable/disable. This is a UX enhancement that can be added incrementally.
