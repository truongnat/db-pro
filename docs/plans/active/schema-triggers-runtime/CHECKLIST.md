# S4 — Schema Triggers Runtime Checklist

## Analysis
- [x] current trigger architecture audited
- [x] domain model gaps identified (table_name, timing, definition, enabled, schema)
- [x] PostgreSQL introspection gaps identified
- [x] SQLite introspection gaps identified (tbl_name discarded)
- [x] provider differences documented
- [x] scope/non-goals explicit

## Implementation
- [x] extend `Trigger` domain struct with table_name, timing, definition, enabled, schema
- [x] fix SQLite introspection to populate all fields (parse SQL body)
- [x] fix PostgreSQL introspection to capture table, timing, function body, enabled
- [ ] add trigger CREATE command (SchemaService + Tauri command)
- [ ] add trigger DROP command (SchemaService + Tauri command)
- [ ] add trigger DDL reconstruction in schema_service.rs
- [ ] add trigger enabled/disabled toggle (PostgreSQL only, capability-gate for SQLite)
- [ ] cache invalidation after trigger mutations

## Frontend
- [x] trigger tab in schema object viewer (pre-existing, now wired to introspection data)
- [x] trigger list component (TriggerManager shows existing triggers with badges)
- [x] CREATE trigger dialog (pre-existing form, now functional with introspection)
- [x] DROP trigger with destructive confirmation (pre-existing button)
- [ ] DDL viewer integration for triggers

## Regression tests
- [x] SQLite: CREATE trigger → DML → observe effect → introspect → DROP
- [x] SQLite: trigger with special identifiers
- [x] SQLite: introspection captures all fields
- [ ] PostgreSQL: introspection source verification
- [ ] Frontend: trigger list rendering test

## Provider/runtime evidence
- [x] SQLite trigger CREATE/DROP/introspect runtime test PASS (5 integration tests)
- [ ] PostgreSQL trigger introspection verified against live provider
- [ ] PostgreSQL trigger CREATE/DROP verified against live provider
- [x] CI: Rust checks PASS, Frontend checks PASS
- [ ] Cache invalidation after trigger mutation verified

## Review gate
- [x] Cubic review #1: P1 fixed (pg_trigger join), P2 deferred to S5
- [ ] Cubic re-review (after P1 fix)
- [x] P0 = 0
- [x] P1 = 0 (after fix)
- [x] quality gates actually executed (CI passing)
- [x] VERIFICATION.md updated with observed evidence
