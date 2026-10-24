---
schema_version: 1
id: "MEM-SRC-dea063672ac49290"
title: "Source mirror: docs/plans/active/issue-438-explain-query-cancellation/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-438-explain-query-cancellation/PLAN.md"
source_sha256: "dea063672ac49290de912dbf7640a5c2eb46c1122707d88832f278c40fac2845"
mirror_body_sha256: "dea063672ac49290de912dbf7640a5c2eb46c1122707d88832f278c40fac2845"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #438 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/438
- Fix: ExplainQuery inserts request_id into query_cancellations and selects on cancel_rx, allowing long-running EXPLAIN ANALYZE queries to be cancelled cleanly.
