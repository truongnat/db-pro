---
schema_version: 1
id: "MEM-SRC-2122cf6c2cec3dce"
title: "Source mirror: docs/plans/active/issue-443-like-escape-wildcards/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-443-like-escape-wildcards/CHECKLIST.md"
source_sha256: "2122cf6c2cec3dce530d87917c1c3ac7f06f957184ba836e16847dc9a17d531a"
mirror_body_sha256: "2122cf6c2cec3dce530d87917c1c3ac7f06f957184ba836e16847dc9a17d531a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #443 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.
