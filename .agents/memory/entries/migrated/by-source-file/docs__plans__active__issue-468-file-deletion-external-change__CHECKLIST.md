---
schema_version: 1
id: "MEM-SRC-50df11f5d32b0165"
title: "Source mirror: docs/plans/active/issue-468-file-deletion-external-change/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-468-file-deletion-external-change/CHECKLIST.md"
source_sha256: "50df11f5d32b01656d6f2e95606b13538c49828e3ad2202189afbdb2fc0af9d8"
mirror_body_sha256: "50df11f5d32b01656d6f2e95606b13538c49828e3ad2202189afbdb2fc0af9d8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #468 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/468
- Fix: check_external_file_changes checks if a file known in workspace_file_mtimes no longer returns an mtime (deleted on disk) and flags it into workspace_external_changes.
