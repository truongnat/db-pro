---
schema_version: 1
id: "MEM-SRC-62d28374b745ecc1"
title: "Source mirror: docs/plans/active/issue-455-er-layout-coordinate-validation/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-455-er-layout-coordinate-validation/VERIFICATION.md"
source_sha256: "62d28374b745ecc1c3df7453018e0d39003e95370d38c051407638069b968812"
mirror_body_sha256: "62d28374b745ecc1c3df7453018e0d39003e95370d38c051407638069b968812"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #455 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/455
- Fix: Sanitizes coordinates in restore_diagram_layouts and apply_position_overrides, rejecting NaN, Inf, and coordinates > 1,000,000 to prevent canvas blanking.
