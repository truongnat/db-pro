---
schema_version: 1
id: "MEM-SRC-63d50a25496ea96d"
title: "Source mirror: docs/plans/active/issue-470-shift-tab-selection-anchor/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-470-shift-tab-selection-anchor/FINDINGS.md"
source_sha256: "63d50a25496ea96d4db11c823a5afcbaf76fbaa924a6da48e2c93cd91b53dafc"
mirror_body_sha256: "63d50a25496ea96d4db11c823a5afcbaf76fbaa924a6da48e2c93cd91b53dafc"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #470 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/470
- Fix: Shift-Tab unindent only shifts anchor/cursor by bytes actually removed on that line, avoiding spurious anchor drift when lines aren't indented.
