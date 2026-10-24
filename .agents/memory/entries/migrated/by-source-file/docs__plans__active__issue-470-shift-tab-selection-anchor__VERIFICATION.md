---
schema_version: 1
id: "MEM-SRC-24860199800a2794"
title: "Source mirror: docs/plans/active/issue-470-shift-tab-selection-anchor/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-470-shift-tab-selection-anchor/VERIFICATION.md"
source_sha256: "24860199800a27947aba9d5a7407b71ec4ce141b8c21d199dfc9c443d7787148"
mirror_body_sha256: "24860199800a27947aba9d5a7407b71ec4ce141b8c21d199dfc9c443d7787148"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #470 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/470
- Fix: Shift-Tab unindent only shifts anchor/cursor by bytes actually removed on that line, avoiding spurious anchor drift when lines aren't indented.
