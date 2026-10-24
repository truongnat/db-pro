---
schema_version: 1
id: "MEM-SRC-978a97a75c37cdd1"
title: "Source mirror: docs/plans/active/issue-455-er-layout-coordinate-validation/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-455-er-layout-coordinate-validation/PLAN.md"
source_sha256: "978a97a75c37cdd1902e493a5fe8b9c97645ff67d2b77b0b2dd15edc11df5ef5"
mirror_body_sha256: "978a97a75c37cdd1902e493a5fe8b9c97645ff67d2b77b0b2dd15edc11df5ef5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #455 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/455
- Fix: Sanitizes coordinates in restore_diagram_layouts and apply_position_overrides, rejecting NaN, Inf, and coordinates > 1,000,000 to prevent canvas blanking.
