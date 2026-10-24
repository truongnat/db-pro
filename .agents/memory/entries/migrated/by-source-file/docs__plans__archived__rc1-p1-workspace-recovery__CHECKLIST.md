---
schema_version: 1
id: "MEM-SRC-d202ff29ae1f10a0"
title: "Source mirror: docs/plans/archived/rc1-p1-workspace-recovery/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/rc1-p1-workspace-recovery/CHECKLIST.md"
source_sha256: "d202ff29ae1f10a0a867a6d1d382844c97e96412b8119edf58f09ba482f931c4"
mirror_body_sha256: "d202ff29ae1f10a0a867a6d1d382844c97e96412b8119edf58f09ba482f931c4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — RC1 Wave 3: Workspace Recovery

- [x] Create plan documentation (`PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`)
- [x] Verify `reassignTabConnection` clears staged grid edits
- [x] Verify `reassignTabConnection` normalizes `schema` between PostgreSQL (`public`) and SQLite (`main`)
- [x] Verify `reassignTabConnection` resets query context and status while preserving SQL text
- [x] Verify `OrphanedTabView` close button routes through `requestCloseTab`
- [x] Add unit & regression tests in `workspace-reassign.test.ts`
- [x] Run all frontend quality gates (`npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`, `npm run test`, `npm run build`)
