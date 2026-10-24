---
schema_version: 1
id: "MEM-SRC-1ce15b2d1b11b907"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/context.md"
source_sha256: "1ce15b2d1b11b9074fe166d2a60de218d78234b169e0d672e73f64ea20d34d05"
mirror_body_sha256: "1ce15b2d1b11b9074fe166d2a60de218d78234b169e0d672e73f64ea20d34d05"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context — #533

Baseline 76b120724a7542de8b587a002811ddacad28156a. Untyped Describe seeds SQLx SQL-only cache with inferred INT4 before eight-byte Int64 binding. Reuse one conversion dispatch with opt-in actual wire-type hints, unspecified OID0 for domain NULL, prepare_with before Describe/stream. Preserve cleanup/cancel scopes; do not drain unused tail to obtain metadata. Review corrections and source pointers: FINDINGS.md and VERIFICATION.md.
