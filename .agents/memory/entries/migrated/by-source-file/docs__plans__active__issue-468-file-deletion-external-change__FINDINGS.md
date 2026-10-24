---
schema_version: 1
id: "MEM-SRC-95fffb950a23088e"
title: "Source mirror: docs/plans/active/issue-468-file-deletion-external-change/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-468-file-deletion-external-change/FINDINGS.md"
source_sha256: "95fffb950a23088e3c52fb07fee957e18844d0f5603b87785a91a60c16fd98d1"
mirror_body_sha256: "95fffb950a23088e3c52fb07fee957e18844d0f5603b87785a91a60c16fd98d1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #468 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/468
- Fix: check_external_file_changes checks if a file known in workspace_file_mtimes no longer returns an mtime (deleted on disk) and flags it into workspace_external_changes.
