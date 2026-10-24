---
schema_version: 1
id: "MEM-PLAN-issue-496-497-fk-dropdown-7b25e477500e"
title: "Plan archive: Issue 496 497 Fk Dropdown"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-496-497-fk-dropdown]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "7b25e477500e01d19c855c0e2a69dedd54c6795da1cc59a60ea9d6043c43044b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-496-497-fk-dropdown/PLAN.md"
  - "docs/plans/active/issue-496-497-fk-dropdown/FINDINGS.md"
  - "docs/plans/active/issue-496-497-fk-dropdown/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-496-497-fk-dropdown/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — #496 + #497

- `foreign_key_for_column` is shared by the dropdown and `open_referenced_row`;
  gating it directly would have broken composite-FK navigation. The 1:1 gate
  lives in `foreign_key_lookup_sql` (the dropdown path) via `dropdown_applies`.
- `lookup_sql` had no `ORDER BY`/`LIMIT`; the pending lookup runs as a normal
  `UiCommand::RunQuery`, so the cap must live in the SQL text.
- Truncation indicator: `apply_foreign_key_lookup` sets a runtime message
  ("Foreign key values truncated to the first 500 rows") when
  `result.rows.len() >= FK_LOOKUP_LIMIT` — keeps the dropdown functional.
- `referenced_row_filters` unchanged: composite keys still navigate.


## Source: `VERIFICATION.md`

# Verification — #496 + #497

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `app::table_fk_lookup_tests::composite_foreign_key_gets_no_value_dropdown` — new (1→2 and 2→1 shapes).
  - `app::table_fk_lookup_tests::foreign_key_lookup_sql_is_bounded_and_ordered` — new:
    `ORDER BY "id"` + `LIMIT 500`.
  - `app::table_fk_lookup_tests::foreign_key_lookup_reports_truncated_options` — new.
  - `app::table_fk_lookup_tests::foreign_key_lookup_edits_and_opens_referenced_row` — existing
    single-column path still dispatches lookup + navigates the referenced row.

