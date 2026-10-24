---
schema_version: 1
id: "MEM-PLAN-explorer-tree-perf-87927ff8d144"
title: "Plan archive: Explorer Tree Perf"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, explorer-tree-perf]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "87927ff8d144462452355fe0dede4aeb7264ff1a0316efa54d0158babe83f09d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/explorer-tree-perf/PLAN.md"
  - "docs/plans/active/explorer-tree-perf/FINDINGS.md"
  - "docs/plans/active/explorer-tree-perf/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/explorer-tree-perf/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Explorer Tree Perf — Verification

## Commands

```bash
cargo check -p db-pro-ui --all-targets
cargo test -p db-pro-ui schema_matching_table_count -- --nocapture
cargo test -p db-pro-ui explorer_search_matches_table_names -- --nocapture
cargo clippy -p db-pro-ui --all-targets -- -D warnings
```

## Status

| Gate | Result |
|------|--------|
| `cargo check -p db-pro-ui --all-targets` | passed |
| `schema_matching_table_count` unit test | passed |
| `explorer_search_matches_table_names` unit test | passed |
| clippy `-D warnings` | passed |
| Runtime large-schema scroll | pending (app relaunch) |

## Runtime checklist (manual)

1. Connect to a schema with ≥500 tables.
2. Expand connection → schema: Tables should be closed; UI stays responsive.
3. Open Tables: first paint may allocate once; scroll should not hitch from off-screen row widgets.
4. Type a filter: Tables auto-opens, badge shows `matching/total`, truncate hint if >100.
5. Open Views only when needed — no clone while closed.

