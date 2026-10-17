# Verification — #496 + #497

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `app::table_fk_lookup_tests::composite_foreign_key_gets_no_value_dropdown` — new (1→2 and 2→1 shapes).
  - `app::table_fk_lookup_tests::foreign_key_lookup_sql_is_bounded_and_ordered` — new:
    `ORDER BY "id"` + `LIMIT 500`.
  - `app::table_fk_lookup_tests::foreign_key_lookup_reports_truncated_options` — new.
  - `app::table_fk_lookup_tests::foreign_key_lookup_edits_and_opens_referenced_row` — existing
    single-column path still dispatches lookup + navigates the referenced row.
