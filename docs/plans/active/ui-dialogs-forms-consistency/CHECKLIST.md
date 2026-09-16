# Checklist — UI08 Dialog, Form, and Confirmation Consistency

- [x] Standardize query dialogs (destructive confirmation, export, overwrite, save as, unsaved query) in `query_dialogs_view.rs`.
- [x] Standardize connection dialogs and form buttons (delete confirmations, presets, URI import, browse) in `connection_view.rs`.
- [x] Standardize inspector window close buttons in `table_metadata_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Record verification evidence.
