---
schema_version: 1
id: "MEM-SRC-9d5aa22bd782fe2e"
title: "Source mirror: docs/plans/active/sqlite-integrity-maintenance/dod.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-integrity-maintenance/dod.md"
source_sha256: "9d5aa22bd782fe2ea36436a903db8ed3600cd3c0f59b22758db099e8600dacfa"
mirror_body_sha256: "9d5aa22bd782fe2ea36436a903db8ed3600cd3c0f59b22758db099e8600dacfa"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Definition of Done

- [ ] R1: public-port tests/smoke prove unsupported table vacuum, actual ANALYZE stats and attached-schema isolation.
- [ ] R2: real engine PK/trigger/known constraint mapping tests pass; preserve core error envelope consumers.
- [ ] R3: capture fresh connector enforcement and cascade result without fixture PRAGMA contamination.
- [ ] Focused and workspace gates run, failures reported honestly; include baseline/candidate runtime evidence.
- [ ] Commit/push and individually close satisfied issues with exact SHA; leave incomplete #403/full UI acceptance unclaimed.
