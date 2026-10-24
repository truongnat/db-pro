---
schema_version: 1
id: "MEM-SRC-db1e17f7b8666ce7"
title: "Source mirror: docs/plans/active/issue-470-shift-tab-selection-anchor/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-470-shift-tab-selection-anchor/CHECKLIST.md"
source_sha256: "db1e17f7b8666ce70ca18e432011c90781aaf8f4f3a6bb918bed4bad34ebc204"
mirror_body_sha256: "db1e17f7b8666ce70ca18e432011c90781aaf8f4f3a6bb918bed4bad34ebc204"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #470 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/470
- Fix: Shift-Tab unindent only shifts anchor/cursor by bytes actually removed on that line, avoiding spurious anchor drift when lines aren't indented.
