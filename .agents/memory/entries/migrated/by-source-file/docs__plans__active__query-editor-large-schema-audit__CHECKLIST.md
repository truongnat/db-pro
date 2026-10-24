---
schema_version: 1
id: "MEM-SRC-7570d64b78fee557"
title: "Source mirror: docs/plans/active/query-editor-large-schema-audit/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/query-editor-large-schema-audit/CHECKLIST.md"
source_sha256: "7570d64b78fee5575fc90134c2e25cd2ddf164d45ab67d7b15161d8ac71e8746"
mirror_body_sha256: "7570d64b78fee5575fc90134c2e25cd2ddf164d45ab67d7b15161d8ac71e8746"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Confirm PostgreSQL and Query-wide symptoms with owner.
- [x] Trace schema load, symbol index, completion generation and popup, result presentation.
- [x] Verify audited paths equal exact HEAD; hash source and binary artifacts.
- [x] Run real optimized provider/index/clone probes at 50/500/1000 tables and 40 columns/table.
- [x] Reproduce Unicode completion panic in actual provider twice across probe iterations.
- [x] Separate measured costs, source-only findings and unverified company crash cause.
- [x] Record local lessons and canonical handoff; global memory unchanged.
- [ ] Reproduce actual company startup crash with redacted backtrace; exact exit vs hang unresolved.
- [ ] Measure native popup frame-time and allocator trace against wide real metadata/results.
- [ ] Implement and verify fixes in a subsequent authorized task; no product patch in this audit.
