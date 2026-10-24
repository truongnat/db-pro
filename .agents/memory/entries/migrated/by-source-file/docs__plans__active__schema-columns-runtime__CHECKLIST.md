---
schema_version: 1
id: "MEM-SRC-a63158a1c1cf039a"
title: "Source mirror: docs/plans/active/schema-columns-runtime/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-columns-runtime/CHECKLIST.md"
source_sha256: "a63158a1c1cf039a7bd0b44305aa1e9bc729a78e1cc253069362fdf6dc58a1e7"
mirror_body_sha256: "a63158a1c1cf039a7bd0b44305aa1e9bc729a78e1cc253069362fdf6dc58a1e7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# S1 — Schema Columns Runtime Checklist

## Source/implementation
- [x] deterministic mutation classification exists
- [x] rename/type/nullability/default SQL generation exists
- [x] batch execution path exists
- [x] PostgreSQL transaction path exists
- [x] SQLite transaction path exists
- [x] schema cache invalidation covers introspect/tableInfo/tableDdl/dependencies/catalog
- [x] compact diff-oriented edit UI exists

## Automated regression
- [ ] middle-statement failure proves rollback/no partial persistence
- [ ] supported SQLite column mutation paths have regression coverage
- [ ] unsupported SQLite mutations are capability-gated and tested

## Live provider runtime
- [ ] PostgreSQL rename + type + nullable combined mutation
- [ ] PostgreSQL failure path proves rollback
- [ ] SQLite support/capability matrix exercised against a live connection

## UI/cache runtime
- [ ] tableInfo refreshes after mutation
- [ ] DDL viewer reflects the new table definition
- [ ] dependencies reflect renamed columns where applicable
- [ ] no stale catalog state after mutation

## Review gate
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] VERIFICATION.md contains actual evidence for all required runtime items
