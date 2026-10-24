---
schema_version: 1
id: "MEM-SRC-d2bd236bfe25e3b9"
title: "Source mirror: docs/plans/active/issue-471-set-text-clears-undo/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-471-set-text-clears-undo/PLAN.md"
source_sha256: "d2bd236bfe25e3b91be5e78f7e5cde7375ab673378dd980c33e30ebf23dc620c"
mirror_body_sha256: "d2bd236bfe25e3b91be5e78f7e5cde7375ab673378dd980c33e30ebf23dc620c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #471 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/471
- Fix: QueryDocument::set_text uses set_text_initial to reset undo history on external buffer reload.
