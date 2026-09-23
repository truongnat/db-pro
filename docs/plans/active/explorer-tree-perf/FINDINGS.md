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
