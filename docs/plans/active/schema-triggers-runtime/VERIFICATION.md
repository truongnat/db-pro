# S4 — Schema Triggers Runtime Verification

State: COMPLETE (all implementation, frontend, and test items done; CI passing)

## CI Evidence

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
| PostgreSQL | DONE (source-level) | existing | existing | DONE (reconstructed from parts) | DONE (ALTER TABLE ENABLE/DISABLE TRIGGER) |
| SQLite | DONE (source + test) | existing | existing | DONE (full SQL from sqlite_master) | N/A (no disable mechanism) |

## DDL viewer integration

`get_table_ddl` now appends trigger definitions after the CREATE TABLE statement.
- SQLite: uses full CREATE TRIGGER SQL from `sqlite_master`
- PostgreSQL: reconstructs from timing/event/action_statement parts

## Trigger enable/disable toggle

- Frontend TriggerRow shows Enable/Disable button for PostgreSQL connections
- Capability-gated via `supportsTriggerToggle` in DdlCapabilities
- SQLite connections do not show the toggle (not supported)
