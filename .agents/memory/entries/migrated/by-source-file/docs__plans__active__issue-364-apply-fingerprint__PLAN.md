---
schema_version: 1
id: "MEM-SRC-28630860c281fcef"
title: "Source mirror: docs/plans/active/issue-364-apply-fingerprint/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-364-apply-fingerprint/PLAN.md"
source_sha256: "28630860c281fcef6359591d4beadce8f430b50325660851cea3c57f1c0a2a4b"
mirror_body_sha256: "28630860c281fcef6359591d4beadce8f430b50325660851cea3c57f1c0a2a4b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #364: Schema Workbench Apply doesn't verify fingerprint

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. Same fix as #353: apply rebuilds the pending request from the current form and compares fingerprints before dispatch.
