# Checklist — #496 + #497

- [x] `dropdown_applies` = 1:1 columns only; composite → `None` (text edit)
- [x] `lookup_sql` emits `ORDER BY <first to_column> LIMIT 500`
- [x] `FK_LOOKUP_LIMIT` constant shared by SQL + truncation note
- [x] `open_referenced_row` still navigates composite FKs
- [x] Tests in `table/tests/table_fk_lookup_tests.rs`
- [x] `cargo test -p db-pro-app`
- [x] `cargo check -p db-pro-app`
