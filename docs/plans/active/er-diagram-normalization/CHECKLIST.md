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
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] CI passing (pending push)
