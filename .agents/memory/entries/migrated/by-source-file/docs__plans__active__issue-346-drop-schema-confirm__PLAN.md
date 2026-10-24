---
schema_version: 1
id: "MEM-SRC-dc571f844a91f47e"
title: "Source mirror: docs/plans/active/issue-346-drop-schema-confirm/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-346-drop-schema-confirm/PLAN.md"
source_sha256: "dc571f844a91f47e02531141fdb0a14828bcf0a5c778188213fac9a246e825fb"
mirror_body_sha256: "dc571f844a91f47e02531141fdb0a14828bcf0a5c778188213fac9a246e825fb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #346: plan_drop_schema opens confirm even when plan fails / SQL empty

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `plan_drop_schema` (schema_workbench_actions.rs:56-63) now sets `apply_confirmation = true` only when `preview_error.is_none() && !preview_sql.trim().is_empty()`.
