---
schema_version: 1
id: "MEM-SRC-8ce95909dd32e357"
title: "Source mirror: docs/plans/active/schema-indexes-runtime/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-indexes-runtime/CHECKLIST.md"
source_sha256: "8ce95909dd32e35722ace1f93f112e7d3aeb0283778c35b298c4040a8fc4b51d"
mirror_body_sha256: "8ce95909dd32e35722ace1f93f112e7d3aeb0283778c35b298c4040a8fc4b51d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# S2 — Schema Indexes Runtime Checklist

## Source/implementation
- [x] index introspection exists for PostgreSQL
- [x] index introspection exists for SQLite
- [x] create/drop index UI exists
- [x] unique/composite index UI exists
- [x] provider-aware identifier quoting exists
- [x] schema cache invalidation path exists after DDL

## Automated evidence
- [x] SQLite CREATE UNIQUE INDEX introspection
- [x] SQLite composite CREATE INDEX introspection
- [x] SQLite DROP INDEX introspection
- [ ] PostgreSQL create/unique/composite/drop integration evidence

## UI/runtime evidence
- [ ] create index from UI → DB → introspection → refreshed UI
- [ ] drop index from UI → DB → introspection → refreshed UI
- [ ] PostgreSQL live provider matrix recorded
- [ ] SQLite UI round-trip recorded

## Review gate
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] VERIFICATION.md contains all required runtime evidence
