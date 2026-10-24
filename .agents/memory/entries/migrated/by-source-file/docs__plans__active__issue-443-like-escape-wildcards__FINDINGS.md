---
schema_version: 1
id: "MEM-SRC-010d112d3116028a"
title: "Source mirror: docs/plans/active/issue-443-like-escape-wildcards/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-443-like-escape-wildcards/FINDINGS.md"
source_sha256: "010d112d3116028a86f439672b617c118cb7b3e5e9694fb573149a08b3083ac1"
mirror_body_sha256: "010d112d3116028a86f439672b617c118cb7b3e5e9694fb573149a08b3083ac1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #443 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.
