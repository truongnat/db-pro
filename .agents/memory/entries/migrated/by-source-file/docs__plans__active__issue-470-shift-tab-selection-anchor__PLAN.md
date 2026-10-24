---
schema_version: 1
id: "MEM-SRC-f10abfe6360eced0"
title: "Source mirror: docs/plans/active/issue-470-shift-tab-selection-anchor/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-470-shift-tab-selection-anchor/PLAN.md"
source_sha256: "f10abfe6360eced0b7edacc945f6eb82c213cba5e7807d59cb91ebfcc762db8d"
mirror_body_sha256: "f10abfe6360eced0b7edacc945f6eb82c213cba5e7807d59cb91ebfcc762db8d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #470 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/470
- Fix: Shift-Tab unindent only shifts anchor/cursor by bytes actually removed on that line, avoiding spurious anchor drift when lines aren't indented.
