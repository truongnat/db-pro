# Findings — #487

- `format_cell_delimited` (`result_grid_export.rs`) returned `String::new()` for
  `UiCell::Null` — identical to `Text("")` → lossy paste/export. Fixed to emit
  `\N`, the Postgres `\copy` NULL sentinel; `format_cell_copy_field` already used
  `\N` for the `COPY`-format export path.
- `copy_cell_at` (`result_grid_clipboard.rs`) early-returned "Empty value:
  nothing to copy" whenever `cell_text` was empty. `copy_cell_value` already
  returns `Some(Text(""))` fine — the gate lived in the copy action. NULL now
  copies as `\N`; empty text copies as `""`.
- `copied_row_text` picks up the sentinel for free via `format_cell_delimited`.
- Tests updated: `app_tests.rs::test_format_cell_csv_and_cell_to_json`,
  `result_grid_record_tests.rs::record_click_copies_null_and_empty_values`
  (renamed; record-view clicks go through `copy_cell_at`).
- Import side: `csv_cell_from_display(.., treat_empty_as_null_token)` is a
  provider-side display toggle, not a `\N` parser — out of scope.
