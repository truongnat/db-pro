---
schema_version: 1
id: "MEM-SRC-59a3e8359f84302b"
title: "Source mirror: docs/plans/active/issue-446-save-query-version-check/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-446-save-query-version-check/CHECKLIST.md"
source_sha256: "59a3e8359f84302bf4fc70857179106edaf3dca9f8f066ca4fb13421c9a5b054"
mirror_body_sha256: "59a3e8359f84302bf4fc70857179106edaf3dca9f8f066ca4fb13421c9a5b054"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #446 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.
