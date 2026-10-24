---
schema_version: 1
id: "MEM-SRC-628240c99cd5f505"
title: "Source mirror: docs/plans/archived/qa-p1-10-orphan-tab-close-guard/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p1-10-orphan-tab-close-guard/CHECKLIST.md"
source_sha256: "628240c99cd5f505949d1fb1ed3d50c6d0d856881a9572996afdfb3f3b36e8da"
mirror_body_sha256: "628240c99cd5f505949d1fb1ed3d50c6d0d856881a9572996afdfb3f3b36e8da"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — QA-P1-10 Orphan Tab Dirty Close Guard

- [x] Create feature plan documentation under `docs/plans/active/qa-p1-10-orphan-tab-close-guard/`
- [x] Verify `OrphanedTabView` in `frontend/src/commons/components/workspace-content.tsx` invokes `requestCloseTab(tabId)`
- [x] Add unit tests in `frontend/src/commons/__tests__/request-close-tab.test.ts`
- [x] Run all frontend quality gates (`typecheck`, `lint`, `format:check`, `check:tokens`, `test`, `build`)
- [ ] Create branch `fix/qa-p1-10-orphan-tab-close-guard` and publish Pull Request
