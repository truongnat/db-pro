---
schema_version: 1
id: "MEM-SRC-8224f9d7902acd41"
title: "Source mirror: docs/plans/active/issue-438-explain-query-cancellation/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-438-explain-query-cancellation/FINDINGS.md"
source_sha256: "8224f9d7902acd41ccba6e319412e0029f5525fe9ebe47f3f942846a0e0e1dee"
mirror_body_sha256: "8224f9d7902acd41ccba6e319412e0029f5525fe9ebe47f3f942846a0e0e1dee"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #438 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/438
- Fix: ExplainQuery inserts request_id into query_cancellations and selects on cancel_rx, allowing long-running EXPLAIN ANALYZE queries to be cancelled cleanly.
