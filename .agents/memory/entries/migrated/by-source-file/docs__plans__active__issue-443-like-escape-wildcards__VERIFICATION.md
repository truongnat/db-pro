---
schema_version: 1
id: "MEM-SRC-fbee90a5f17bed8a"
title: "Source mirror: docs/plans/active/issue-443-like-escape-wildcards/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-443-like-escape-wildcards/VERIFICATION.md"
source_sha256: "fbee90a5f17bed8a9a79b02821834a0571bfac0f5818d47f77ce9815adf71b73"
mirror_body_sha256: "fbee90a5f17bed8a9a79b02821834a0571bfac0f5818d47f77ce9815adf71b73"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #443 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.
