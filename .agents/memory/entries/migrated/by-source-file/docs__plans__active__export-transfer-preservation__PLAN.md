---
schema_version: 1
id: "MEM-SRC-6fd3550bce7ed045"
title: "Source mirror: docs/plans/active/export-transfer-preservation/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/export-transfer-preservation/PLAN.md"
source_sha256: "6fd3550bce7ed0450bc73c4a3064302d7f099da9c079853f3b87af421f450a16"
mirror_body_sha256: "6fd3550bce7ed0450bc73c4a3064302d7f099da9c079853f3b87af421f450a16"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — Safe Transfer Cleanup and Empty CSV Headers

## Objective

Fix related export/transfer issues #436 and #437 without deleting a prior successful export or publishing a schema-less empty CSV/TSV.

## Target Branch

`fix/export-transfer-preserves-destination`

## Severity

P1 data-loss regression (#436) and P2 empty delimited export schema loss (#437).

## Scope

- Restrict cleanup of CSV/TSV, JSONL, and Excel targets to the current run's temporary output.
- Preserve an existing final destination on cancel/error.
- Write delimited headers during finish when no rows were written.
- Keep JSONL's standard record-only representation: an empty export has zero lines; it has no header-record convention.
