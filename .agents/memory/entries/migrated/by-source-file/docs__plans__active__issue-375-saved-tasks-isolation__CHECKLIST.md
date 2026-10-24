---
schema_version: 1
id: "MEM-SRC-f88d656d00b835e4"
title: "Source mirror: docs/plans/active/issue-375-saved-tasks-isolation/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-375-saved-tasks-isolation/CHECKLIST.md"
source_sha256: "f88d656d00b835e48441f68d4c6b5be084b4c72a58b30b9675aaae0825fd4cba"
mirror_body_sha256: "f88d656d00b835e48441f68d4c6b5be084b4c72a58b30b9675aaae0825fd4cba"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #375 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/375
- Fix: dispatch_sql_task dispatches directly via UiCommand::RunQuery on the task's connection without opening a document, changing active tabs, or hijacking the user's active editor session.
