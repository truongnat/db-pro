---
schema_version: 1
id: "MEM-SRC-ad86940d6f4be9de"
title: "Source mirror: docs/plans/active/issue-438-explain-query-cancellation/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-438-explain-query-cancellation/CHECKLIST.md"
source_sha256: "ad86940d6f4be9de3512f229179de0d28ec5a38ce602660095ac47416b1f1bda"
mirror_body_sha256: "ad86940d6f4be9de3512f229179de0d28ec5a38ce602660095ac47416b1f1bda"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #438 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/438
- Fix: ExplainQuery inserts request_id into query_cancellations and selects on cancel_rx, allowing long-running EXPLAIN ANALYZE queries to be cancelled cleanly.
