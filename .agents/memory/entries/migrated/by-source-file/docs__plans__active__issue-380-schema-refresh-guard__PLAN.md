---
schema_version: 1
id: "MEM-SRC-9a6fc322095154bf"
title: "Source mirror: docs/plans/active/issue-380-schema-refresh-guard/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-380-schema-refresh-guard/PLAN.md"
source_sha256: "9a6fc322095154bf6bc5a749bf3f2e792a2b23c5e1e08c13ceb81874273a496e"
mirror_body_sha256: "9a6fc322095154bf6bc5a749bf3f2e792a2b23c5e1e08c13ceb81874273a496e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #380: Schema refresh has no in-flight guard

Status: VERIFIED-FIXED (already fixed in 22819b13)

No code change required. `refresh_schema_palette` returns early with 'Schema refresh already in progress…' when `schema.explorer.schema_request.is_some()`. File (palette/palette_view.rs) belongs to a sibling; verified read-only.
