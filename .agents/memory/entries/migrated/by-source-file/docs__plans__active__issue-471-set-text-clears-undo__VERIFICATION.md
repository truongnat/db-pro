---
schema_version: 1
id: "MEM-SRC-e572779e16388e4c"
title: "Source mirror: docs/plans/active/issue-471-set-text-clears-undo/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-471-set-text-clears-undo/VERIFICATION.md"
source_sha256: "e572779e16388e4c58aaa0adb43bbdf141c203f2a191242b696d9c123a2424ff"
mirror_body_sha256: "e572779e16388e4c58aaa0adb43bbdf141c203f2a191242b696d9c123a2424ff"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #471 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/471
- Fix: QueryDocument::set_text uses set_text_initial to reset undo history on external buffer reload.
