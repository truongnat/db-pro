# Verification — #487

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `result_grid::tests::delimited_cells_distinguish_null_from_empty_text` — new.
  - `result_grid::tests::copy_cell_copies_empty_text_and_null_sentinel` — new.
  - `result_grid::tests::copied_row_text_distinguishes_null_from_empty_text` — new.
  - `app::result_grid_view::record_tests::record_click_copies_null_and_empty_values` — updated.
  - `app::tests::test_format_cell_csv_and_cell_to_json`,
    `app::tests::test_export_result_writes_escaped_delimited_text` — updated to `\N`.
