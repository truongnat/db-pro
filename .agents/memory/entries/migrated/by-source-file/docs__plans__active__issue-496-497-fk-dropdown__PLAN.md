---
schema_version: 1
id: "MEM-SRC-a64ddb233dac716b"
title: "Source mirror: docs/plans/active/issue-496-497-fk-dropdown/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-496-497-fk-dropdown/PLAN.md"
source_sha256: "a64ddb233dac716b00d4cf3b7ff23ec549b6e386e9827203c59f25a29a0865a2"
mirror_body_sha256: "a64ddb233dac716b00d4cf3b7ff23ec549b6e386e9827203c59f25a29a0865a2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #496 + #497 — FK cell dropdown: 1:1 keys only, bounded lookup

## Goal
- #496: `foreign_key_for_column` matched any member of `from_columns` and
  `options_from_result` joined all selected row values with ", " → a composite
  FK offered "7, 42" labels that would be written into one column.
- #497: `lookup_sql` emitted unbounded `SELECT cols FROM schema.table` → the
  whole referenced table was fetched on every FK cell edit.

## Scope
- `table_fk_lookup.rs`: `FK_LOOKUP_LIMIT = 500`; `dropdown_applies(fk)` =
  `from_columns.len()==1 && to_columns.len()==1`; `lookup_sql` gains
  `ORDER BY <first to_column>` + `LIMIT 500`.
- `table_mutation_actions.rs`: `foreign_key_lookup_sql` returns `None` when
  `!dropdown_applies(fk)` (composite → plain text edit); `apply_foreign_key_lookup`
  sets a runtime note when the result lands on the cap (possible truncation).
- `open_referenced_row` unchanged — composite-FK navigation still uses
  `referenced_row_filters` over all key parts.

## Non-goals
- Searchable/paginated picker (explicitly out of scope).
- No change to per-cell FK semantics in `options_from_result`.

## Acceptance
- Composite FK cell → `foreign_key_lookup_sql` → `None` (text edit, no dropdown).
- Single-column FK SQL contains `ORDER BY` + `LIMIT 500`.
- `cargo test -p db-pro-app` + `cargo check -p db-pro-app`.
