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
