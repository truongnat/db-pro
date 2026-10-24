---
schema_version: 1
id: "MEM-SRC-a73d8f9fcc1c2d77"
title: "Source mirror: docs/plans/active/issue-346-drop-schema-confirm/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-346-drop-schema-confirm/FINDINGS.md"
source_sha256: "a73d8f9fcc1c2d77171b3fd56006585c27b3706845a2b146e26b07a55b43cf5c"
mirror_body_sha256: "a73d8f9fcc1c2d77171b3fd56006585c27b3706845a2b146e26b07a55b43cf5c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #346

Confirmed the guard exists at schema_workbench_actions.rs:59-63. Failed plans set `preview_error` and clear `preview_sql` in `plan_workbench_action`, so the dialog cannot open for failed/empty plans.
