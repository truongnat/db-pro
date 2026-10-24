---
schema_version: 1
id: "MEM-SRC-6a432b35882370a3"
title: "Source mirror: docs/plans/active/issue-354-database-action-flags/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-354-database-action-flags/PLAN.md"
source_sha256: "6a432b35882370a3bcbee56538095f37aa0b4bf1509893641449e59db38a31f3"
mirror_body_sha256: "6a432b35882370a3bcbee56538095f37aa0b4bf1509893641449e59db38a31f3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #354: plan_database_action sets both if_exists and if_not_exists; cascade leaks into DROP DATABASE

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `plan_database_action` delegates to `build_database_request` which sets `if_exists: action == Drop`, `if_not_exists: action == Create`, `cascade: false`.
