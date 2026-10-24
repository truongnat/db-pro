---
schema_version: 1
id: "MEM-SRC-0a4526e49a58a497"
title: "Source mirror: docs/plans/active/issue-381-workbench-stale-context/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-381-workbench-stale-context/PLAN.md"
source_sha256: "0a4526e49a58a4975469db34b5748a41348229ba0bf81183a02f8a0fc59288ee"
mirror_body_sha256: "0a4526e49a58a4975469db34b5748a41348229ba0bf81183a02f8a0fc59288ee"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #381: open_schema_workbench keeps stale preview/name/confirmation when schema context changes

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `open_schema_workbench` detects schema/table context change and calls `clear_plan_state()` + clears `name`.
