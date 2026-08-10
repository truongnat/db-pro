# S4 — Schema Triggers Runtime Checklist

## Analysis
- [x] current trigger architecture audited
- [x] domain model gaps identified (table_name, timing, definition, enabled, schema)
- [x] PostgreSQL introspection gaps identified
- [x] SQLite introspection gaps identified (tbl_name discarded)
- [x] provider differences documented
- [x] scope/non-goals explicit

## Implementation
- [ ] extend `Trigger` domain struct with table_name, timing, definition, enabled, schema
- [ ] fix SQLite introspection to populate all fields (parse SQL body)
- [ ] fix PostgreSQL introspection to capture table, timing, function body, enabled
- [ ] add trigger CREATE command (SchemaService + Tauri command)
- [ ] add trigger DROP command (SchemaService + Tauri command)
- [ ] add trigger DDL reconstruction in schema_service.rs
- [ ] add trigger enabled/disabled toggle (PostgreSQL only, capability-gate for SQLite)
- [ ] cache invalidation after trigger mutations

## Frontend
- [ ] trigger tab in schema object viewer
- [ ] trigger list component
- [ ] CREATE trigger dialog
- [ ] DROP trigger with destructive confirmation
- [ ] DDL viewer integration for triggers

## Regression tests
- [ ] SQLite: CREATE trigger → DML → observe effect → introspect → DROP
- [ ] SQLite: trigger with special identifiers
- [ ] SQLite: introspection captures all fields
- [ ] PostgreSQL: introspection source verification
- [ ] Frontend: trigger list rendering test

## Provider/runtime evidence
- [ ] SQLite trigger CREATE/DROP/introspect runtime test PASS
- [ ] PostgreSQL trigger introspection verified against live provider
- [ ] PostgreSQL trigger CREATE/DROP verified against live provider
- [ ] UI trigger tab displays correct data
- [ ] Cache invalidation after trigger mutation verified

## Review gate
- [ ] independent reviewer result recorded
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] quality gates actually executed
- [ ] VERIFICATION.md updated with observed evidence
