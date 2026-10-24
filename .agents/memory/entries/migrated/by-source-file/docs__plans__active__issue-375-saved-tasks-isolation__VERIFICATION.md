---
schema_version: 1
id: "MEM-SRC-010dbbcf36d7ae6f"
title: "Source mirror: docs/plans/active/issue-375-saved-tasks-isolation/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-375-saved-tasks-isolation/VERIFICATION.md"
source_sha256: "010dbbcf36d7ae6fd47fb57ac98dbf1fda1a971527aae3b72a2ea4803bf6ea72"
mirror_body_sha256: "010dbbcf36d7ae6fd47fb57ac98dbf1fda1a971527aae3b72a2ea4803bf6ea72"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #375 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/375
- Fix: dispatch_sql_task dispatches directly via UiCommand::RunQuery on the task's connection without opening a document, changing active tabs, or hijacking the user's active editor session.
