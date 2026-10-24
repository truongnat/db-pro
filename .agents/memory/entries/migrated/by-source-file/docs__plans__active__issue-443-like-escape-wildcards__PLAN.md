---
schema_version: 1
id: "MEM-SRC-3d7a8acc70145eba"
title: "Source mirror: docs/plans/active/issue-443-like-escape-wildcards/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-443-like-escape-wildcards/PLAN.md"
source_sha256: "3d7a8acc70145eba770a5c16bc289828741efc368c9dcbdd7a75655ec4e02b5e"
mirror_body_sha256: "3d7a8acc70145eba770a5c16bc289828741efc368c9dcbdd7a75655ec4e02b5e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #443 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.
