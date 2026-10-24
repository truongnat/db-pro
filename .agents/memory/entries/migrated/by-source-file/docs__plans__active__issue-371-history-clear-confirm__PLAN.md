---
schema_version: 1
id: "MEM-SRC-4a2098d57ff0a41b"
title: "Source mirror: docs/plans/active/issue-371-history-clear-confirm/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-371-history-clear-confirm/PLAN.md"
source_sha256: "4a2098d57ff0a41b85dd3abac9db961a9d597ee7ac64ce63c8b6a1597cb8987e"
mirror_body_sha256: "4a2098d57ff0a41b85dd3abac9db961a9d597ee7ac64ce63c8b6a1597cb8987e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #371: DDL history Clear has no confirm; Open-in-editor spams documents

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. Two-step confirm (`history_clear_confirm` + Confirm clear / Keep history) in draw_execution_history; `OpenSqlAsQuery` reuses a document already holding the same SQL or an empty active doc.
