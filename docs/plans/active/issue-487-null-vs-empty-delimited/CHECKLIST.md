# Checklist — #487

- [x] `UiCell::Null` → `\N` in `format_cell_delimited` (CSV + TSV + row copy)
- [x] `Text("")` still an empty field; escaping/quoting unchanged
- [x] `copy_cell_at`: empty string copyable; NULL copies `\N`
- [x] Tests: `result_grid_export_tests`, `result_grid_clipboard_tests`, updated
      `app_tests` + `result_grid_record_tests`
- [x] `cargo test -p db-pro-app`
- [x] `cargo check -p db-pro-app`
