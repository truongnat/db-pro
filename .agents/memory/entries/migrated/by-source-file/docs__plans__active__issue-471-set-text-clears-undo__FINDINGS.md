---
schema_version: 1
id: "MEM-SRC-b209439675969087"
title: "Source mirror: docs/plans/active/issue-471-set-text-clears-undo/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-471-set-text-clears-undo/FINDINGS.md"
source_sha256: "b2094396759690879ecc3e0096bf11190800f554e2b1b1cee63d7d7a7e4e1c08"
mirror_body_sha256: "b2094396759690879ecc3e0096bf11190800f554e2b1b1cee63d7d7a7e4e1c08"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #471 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/471
- Fix: QueryDocument::set_text uses set_text_initial to reset undo history on external buffer reload.
