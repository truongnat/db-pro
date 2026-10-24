---
schema_version: 1
id: "MEM-SRC-82e81b64f1535914"
title: "Source mirror: docs/plans/active/issue-471-set-text-clears-undo/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-471-set-text-clears-undo/CHECKLIST.md"
source_sha256: "82e81b64f15359148b43697f532b25b739bb76bbfdc6163960f9b876de3c30f4"
mirror_body_sha256: "82e81b64f15359148b43697f532b25b739bb76bbfdc6163960f9b876de3c30f4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #471 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/471
- Fix: QueryDocument::set_text uses set_text_initial to reset undo history on external buffer reload.
