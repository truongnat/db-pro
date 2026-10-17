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
