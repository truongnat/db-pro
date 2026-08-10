# S4 — Schema Triggers Runtime Verification

State: IMPLEMENTING (domain + introspection + tests complete; CI passing)

## CI Evidence

| Check | Run | Result |
|---|---|---|
| Rust checks (cargo fmt + cargo test) | [31417605180](https://github.com/truongnat/db-pro/actions/runs/31417605180) | PASS |
| Frontend checks (typecheck + lint + test + build) | [31417605180](https://github.com/truongnat/db-pro/actions/runs/31417605180) | PASS |
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

| Provider | Introspection | CREATE | DROP | DDL | Enabled/Disabled |
|---|---|---|---|---|---|
| PostgreSQL | DONE (source-level) | existing | existing | existing | DONE (pg_trigger.tgenabled) |
| SQLite | DONE (source + test) | existing | existing | existing | always true (no disable mechanism) |
