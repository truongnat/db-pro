---
schema_version: 1
id: "MEM-SRC-70f51e25b847ff44"
title: "Source mirror: docs/plans/active/welcome-query-editor-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/welcome-query-editor-polish/CHECKLIST.md"
source_sha256: "70f51e25b847ff442df818298c3f900bdd398cb188b8e50b928fb7d4d8fb7141"
mirror_body_sha256: "70f51e25b847ff442df818298c3f900bdd398cb188b8e50b928fb7d4d8fb7141"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Welcome and query editor polish — Checklist

## Planning
- [x] Evidence and failure scenario recorded
- [x] Scope/non-goals explicit
- [x] PostgreSQL/SQLite support matrix explicit

## Implementation
- [x] Smallest coherent implementation complete
- [x] Safety/capability rules respected
- [x] Cache/state invalidation accounted for (n/a — presentation only)

## Tests
- [x] `welcome_top_inset_keeps_the_start_block_in_the_upper_third` — **[NOTE 2026-10-13]** test no longer exists at HEAD (Welcome redesigned `3ea9e249`..`c6a033da`); retained as historical record of the run, see VERIFICATION audit update
- [x] `the_filter_field_border_is_painted_inside_the_field` still passes with a connection present
- [x] PostgreSQL: N/A (no SQL change)
- [x] SQLite: N/A (no SQL change)

## Review
- [x] Architecture review complete (self)
- [x] Correctness/security review complete (self)
- [x] P0 = 0
- [x] P1 = 0

## Runtime
- [x] Capture screenshots of Welcome and Query at the three gate sizes
- [x] VERIFICATION.md contains the commands actually run
- [x] STATUS.md row added
