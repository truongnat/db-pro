# Checklist — UI07 Table/Object/Schema Workbench Quality

- [x] Standardize schema workbench planning action buttons in `schema_workbench_form.rs`.
- [x] Standardize schema object workbench actions (routine source editor, execution, DDL preview, drop confirmation) in `schema_object_view.rs`.
- [x] Standardize table DDL tab actions (Refresh, Open in Query, Copy) in `table_ddl_view.rs`.
- [x] Standardize table metadata filter clear and jump actions in `table_metadata_view.rs`.
- [x] Standardize table workspace welcome and toolbar actions in `table_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Record verification evidence.
