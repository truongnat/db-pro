---
schema_version: 1
id: "MEM-SRC-140a3a18fdc0f447"
title: "Source mirror: docs/plans/active/explorer-tree-perf/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/explorer-tree-perf/PLAN.md"
source_sha256: "140a3a18fdc0f447e5ad2b95713b604ae5c8cfcc452031c343281b7a02410278"
mirror_body_sha256: "140a3a18fdc0f447e5ad2b95713b604ae5c8cfcc452031c343281b7a02410278"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Explorer Tree Cache + Render Performance

## Goal

Make the Database Explorer tree responsive on large schemas (1000+ tables)
without changing introspection backend behaviour.

## Problem (evidence)

- Tables folder defaulted open → every frame walked/painted the full table list.
- Per-frame `schema.tables.clone()` + `filter_by_schema(...).cloned()` for
  Views/Functions/Triggers even when those folders were closed.
- No viewport culling; off-screen table rows still ran full row widgets.
- Filter badge path materialised all table name strings just to count matches.

## Scope

- Cache visible explorer table names per `(connection, schema, search)`.
- Default Tables folder closed; auto-open while search is active.
- Defer Views/Functions/Triggers materialisation until the folder is open.
- Viewport-cull table rows; keep layout height for off-screen rows.
- Cap painted rows at `EXPLORER_MAX_TABLES` with a refine-filter hint.
- Schema-scoped folder persistence IDs (avoid cross-schema collapse bleed).

## Non-goals

- Virtualised egui scroll (true windowing) — follow-up if still needed.
- Backend pagination of introspection.
- Changing search semantics.

## Acceptance

- Closed Tables folder: no per-frame table-name Vec allocation for listing.
- Open Tables folder: paint at most `EXPLORER_MAX_TABLES` rows + cull off-screen.
- `cargo check -p db-pro-ui`, targeted tests, and clippy on touched crates pass.
- Runtime: expand a 1k-table schema without obvious UI hitch on open/scroll.
