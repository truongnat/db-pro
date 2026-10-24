---
schema_version: 1
id: "MEM-SRC-1769533afe06591f"
title: "Source mirror: docs/plans/active/explorer-tree-perf/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/explorer-tree-perf/FINDINGS.md"
source_sha256: "1769533afe06591f848e515ee5405dec4bc9445d3a00af838e7e5027f8de2d39"
mirror_body_sha256: "1769533afe06591f848e515ee5405dec4bc9445d3a00af838e7e5027f8de2d39"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Explorer Tree Perf — Findings

## P1 — Per-frame full table list work

`draw_dbeaver_schema_objects` previously called `active_schema_table_names()`
(clone of every table name) and opened Tables by default, so a ~1100-table
schema paid full filter + row widget cost every frame.

## P1 — Eager folder filters

Views/Functions/Triggers ran `filter_by_schema(...).cloned()` every frame
regardless of collapse state.

## P2 — Shared folder IDs across schemas

`codex_views_folder` (etc.) was not schema-scoped, so expand state leaked
across PostgreSQL schemas.

## Fix shape

Cache + defer + cull + default-closed Tables; keep backend unchanged.
