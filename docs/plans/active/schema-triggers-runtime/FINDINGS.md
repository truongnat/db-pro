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
Pending Cubic review on PR #8.
