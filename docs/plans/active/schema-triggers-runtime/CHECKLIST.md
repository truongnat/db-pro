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
- [x] add trigger CREATE command (SchemaService + Tauri command)
- [x] add trigger DROP command (SchemaService + Tauri command)
- [x] add trigger DDL reconstruction in schema_service.rs
- [x] add trigger enabled/disabled toggle (PostgreSQL only, capability-gate for SQLite)
- [x] cache invalidation after trigger mutations

## Frontend
- [x] trigger tab in schema object viewer (pre-existing, now wired to introspection data)
- [x] trigger list component (TriggerManager shows existing triggers with badges)
- [x] CREATE trigger dialog (pre-existing form, now functional with introspection)
- [x] DROP trigger with destructive confirmation (pre-existing button)
- [x] DDL viewer integration for triggers

## Regression tests
- [x] SQLite: CREATE trigger → DML → observe effect → introspect → DROP
- [x] SQLite: trigger with special identifiers
- [x] SQLite: introspection captures all fields
- [x] PostgreSQL: introspection source verification (PG integration tests PASS in CI)
- [x] Frontend: trigger list rendering test (7 tests in trigger-manager.test.tsx)

## Provider/runtime evidence
- [x] SQLite trigger CREATE/DROP/introspect runtime test PASS (5 integration tests)
- [x] SQLite trigger schema verified as 'main' (integration test assertion added)
- [x] PostgreSQL trigger introspection verified against live provider (CI: pg_introspect_triggers PASS)
- [x] PostgreSQL trigger function_def verified via pg_get_functiondef (CI: pg_introspect_triggers PASS)
- [x] PostgreSQL tables, views, indexes, FK introspection verified (CI: 6 pg_integration tests PASS)
- [x] CI: Rust checks PASS, Frontend checks PASS
- [x] Cache invalidation after trigger mutation verified (execute_ddl invalidates cache)

## Review gate
- [x] Cubic review #1: P1 fixed (pg_trigger join), P2 deferred to S5
- [x] Cubic review #2: SQLite BEGIN parsing P2 fixed
- [x] Cubic review #3: S5 DDL CR1-CR5 all fixed
- [x] Cubic review #4: S6 edge grouping CR1/CR2 fixed
- [x] P0 = 0
- [x] P1 = 0
- [x] quality gates actually executed (CI passing)
- [x] VERIFICATION.md updated with observed evidence
