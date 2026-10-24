---
schema_version: 1
id: "MEM-SRC-615d558453ea3884"
title: "Source mirror: docs/plans/active/schema-regression/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-regression/CHECKLIST.md"
source_sha256: "615d558453ea38845cf1ca8107e092db44f5cd00c6aeb7231f867b8d163976ae"
mirror_body_sha256: "615d558453ea38845cf1ca8107e092db44f5cd00c6aeb7231f867b8d163976ae"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
