---
schema_version: 1
id: "MEM-SRC-626da291950b986b"
title: "Source mirror: docs/plans/active/issue-375-saved-tasks-isolation/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-375-saved-tasks-isolation/PLAN.md"
source_sha256: "626da291950b986b9ad8a759786feba6da0d38278bc915ced4fb8c3bd953529b"
mirror_body_sha256: "626da291950b986b9ad8a759786feba6da0d38278bc915ced4fb8c3bd953529b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #375 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/375
- Fix: dispatch_sql_task dispatches directly via UiCommand::RunQuery on the task's connection without opening a document, changing active tabs, or hijacking the user's active editor session.
