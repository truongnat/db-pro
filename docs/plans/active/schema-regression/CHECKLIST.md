# S7 — Full Schema Regression Checklist

## Audit
- [x] Existing Rust test inventory catalogued (integration, pg_integration, schema_indexes, schema_triggers)
- [x] Existing frontend test inventory catalogued (105 files, 1319 tests) **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The frontend was archived on 2026-09-11; this count is historical only and no longer describes any current gate. The original claim is retained, unretracted.
- [x] CI evidence reviewed (PG service container + SQLite in-memory)
- [x] Per-feature coverage gaps identified

## Regression matrix
- [x] S1 Columns matrix complete
- [x] S2 Indexes matrix complete
- [x] S3 Relations matrix complete
- [x] S4 Triggers matrix complete
- [x] S5 DDL matrix complete
- [x] S6 ER Diagram matrix complete

## Gap-filling tests
- [x] Add PG composite FK detail test (column mapping, constraint identity, PK columns)
- [x] Add PG index lifecycle test (CREATE → introspect → DROP → verify)
- [x] Add PG special identifier test (unicode table/column, quoted names, reserved words)

## Schema readiness report
- [x] Release-style readiness report appended to FINDINGS.md
- [x] Per-feature readiness grade assigned
- [x] Blocking gaps documented

## Review gate
- [x] P0 = 0
- [x] P1 = 0
- [x] CI passing
- [x] VERIFICATION.md updated
