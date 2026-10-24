---
schema_version: 1
id: "MEM-SRC-5f88f9abf99a4112"
title: "Source mirror: docs/plans/active/issue-375-saved-tasks-isolation/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-375-saved-tasks-isolation/FINDINGS.md"
source_sha256: "5f88f9abf99a41129cfb4a82eb16c4418f05c2f2ade80fc035d95de8278a2e3b"
mirror_body_sha256: "5f88f9abf99a41129cfb4a82eb16c4418f05c2f2ade80fc035d95de8278a2e3b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #375 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/375
- Fix: dispatch_sql_task dispatches directly via UiCommand::RunQuery on the task's connection without opening a document, changing active tabs, or hijacking the user's active editor session.
