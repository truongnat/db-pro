---
schema_version: 1
id: "MEM-SRC-b01e3ff5789b4899"
title: "Source mirror: docs/plans/active/schema-indexes-runtime/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-indexes-runtime/FINDINGS.md"
source_sha256: "b01e3ff5789b4899a985c672c6c3e4e1f0ef81553bfaccb45761fb47619472b1"
mirror_body_sha256: "b01e3ff5789b4899a985c672c6c3e4e1f0ef81553bfaccb45761fb47619472b1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# S2 — Schema Indexes Runtime Findings

## Confirmed implementation

- Index listing/creation/drop surfaces exist in the frontend.
- PostgreSQL and SQLite introspection paths exist.
- SQLite automated verification covers unique, composite and drop index behavior.
- DDL success invalidates schema metadata surfaces in source.

These are implementation/source claims except where the SQLite integration test is explicitly cited.

## Open evidence gaps

### P2-001 — PostgreSQL runtime evidence missing
Status: OPEN.

The merged S2 verification exercised SQLite only. PostgreSQL index introspection and DDL support are present in source, but no actual PostgreSQL runtime result is recorded.

### P2-002 — UI refresh lifecycle not runtime-verified
Status: OPEN.

Source wiring exists for query invalidation, but no recorded runtime evidence proves create/drop from the UI refreshes the visible index list without stale state.

## Rejected wording from previous plan

Claims such as "perfectly supports" and "passed flawlessly" were removed because they exceeded the evidence actually collected.

## P0/P1 gate

No accepted open P0/P1 finding is currently recorded for S2. The feature remains `RUNTIME_VERIFY` because evidence is incomplete, not because an accepted P0/P1 defect is open.
