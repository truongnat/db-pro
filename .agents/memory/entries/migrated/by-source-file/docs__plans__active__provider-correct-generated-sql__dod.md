---
schema_version: 1
id: "MEM-SRC-28656e689477ac0c"
title: "Source mirror: docs/plans/active/provider-correct-generated-sql/dod.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/provider-correct-generated-sql/dod.md"
source_sha256: "28656e689477ac0cd0996dcd1e7c3baf580b6282cc27b94b9863fbdf0796d6bb"
mirror_body_sha256: "28656e689477ac0cd0996dcd1e7c3baf580b6282cc27b94b9863fbdf0796d6bb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Definition of Done — #441/#442

Real SQLite/PostgreSQL/MySQL public PK+DDL round trips; MySQL cross-database FK test; core provider matrix. Live SQLServer remains pending, so #441 stays open despite source/test fix.

CHECKLIST.md tracks actual states; VERIFICATION.md is authoritative for executed commands, failures/skips and runtime limits. A historical worker NOT RUN statement is not a parent PASS.
