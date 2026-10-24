---
schema_version: 1
id: "MEM-SRC-ba68212fe418ca59"
title: "Source mirror: docs/plans/active/issue-353-preview-fingerprint/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-353-preview-fingerprint/PLAN.md"
source_sha256: "ba68212fe418ca598ec74f80793b8b27df18184ebaedd2d80b98d1ebbf219fb9"
mirror_body_sha256: "ba68212fe418ca598ec74f80793b8b27df18184ebaedd2d80b98d1ebbf219fb9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #353: preview_fingerprint stored but never verified before Apply

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `apply_workbench_ddl` calls `replan_fingerprint` and rejects apply when the live fingerprint differs (`Plan is stale — regenerate preview`).
