---
schema_version: 1
id: "MEM-SRC-63dbfa15f649c88d"
title: "Source mirror: docs/plans/active/issue-468-file-deletion-external-change/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-468-file-deletion-external-change/PLAN.md"
source_sha256: "63dbfa15f649c88d5305281b547e923d76f6c88ddeea1d6e719c4ca786d02af4"
mirror_body_sha256: "63dbfa15f649c88d5305281b547e923d76f6c88ddeea1d6e719c4ca786d02af4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #468 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/468
- Fix: check_external_file_changes checks if a file known in workspace_file_mtimes no longer returns an mtime (deleted on disk) and flags it into workspace_external_changes.
