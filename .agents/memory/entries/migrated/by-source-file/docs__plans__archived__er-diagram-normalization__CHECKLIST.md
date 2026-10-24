---
schema_version: 1
id: "MEM-SRC-a4dce41b47672bae"
title: "Source mirror: docs/plans/archived/er-diagram-normalization/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/er-diagram-normalization/CHECKLIST.md"
source_sha256: "a4dce41b47672bae36dd0e97116b0a596d43c8e44dfef694911a94389ae44bb3"
mirror_body_sha256: "a4dce41b47672bae36dd0e97116b0a596d43c8e44dfef694911a94389ae44bb3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# S6 — ER Diagram Normalization Checklist

## Analysis
- [x] ER diagram code audited (er-diagram.tsx, table-node.tsx, layout.ts)
- [x] Composite FK issue identified (P1: multiple edges per constraint)
- [x] Edge identity issue identified (P2: sequential index IDs)
- [x] Node identity verified (schema-qualified ✅)
- [x] Column handles verified (PK source, FK target ✅)
- [x] Position persistence verified ✅
- [x] Self-relation support verified ✅

## Implementation
- [x] Group FK entries by constraint name for edge generation (`edge-builder.ts`)
- [x] Use constraint name in edge ID (`fk:${schema}.${name}`)
- [x] Refactored `er-diagram.tsx` to use `groupForeignKeys()` utility
- [x] Self-referencing composite FKs handled (tested)

## Tests
- [x] Composite FK grouping test (2 columns → 1 edge)
- [x] Edge ID stability test (constraint name based)
- [x] Single FK still produces 1 edge
- [x] Self-referencing FK test
- [x] Hidden table exclusion test
- [x] Empty FK list test
- [x] Different constraint names not merged
- [x] Full suite: 1316 tests, 105 files, ALL PASSING

## Review gate
- [x] P0 = 0
- [x] P1 = 0
- [x] CI passing (run 31428343540 — Rust + Frontend green)
