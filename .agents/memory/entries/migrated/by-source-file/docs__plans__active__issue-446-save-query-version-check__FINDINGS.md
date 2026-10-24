---
schema_version: 1
id: "MEM-SRC-0ea661bb31c23a3e"
title: "Source mirror: docs/plans/active/issue-446-save-query-version-check/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-446-save-query-version-check/FINDINGS.md"
source_sha256: "0ea661bb31c23a3ec21f610af86b85cc6f6794950465bec10fb1cc2f53fe4b26"
mirror_body_sha256: "0ea661bb31c23a3ec21f610af86b85cc6f6794950465bec10fb1cc2f53fe4b26"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #446 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.
