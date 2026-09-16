# Checklist — UI06 Data Grid Hardening

- [x] Standardize result grid toolbar actions (`Copy Cell`, `Copy Row`, `CSV`, `JSON`, `Record`, `Inspect`) in `result_grid_view.rs`.
- [x] Standardize table data toolbar actions (`Add Row`, `Apply`, `Discard`, `Reload Row`, `Discard Local Change`, `Resolve Conflict`, `Retry`) in `table_editor_view.rs`.
- [x] Standardize table data pagination controls (`First`, `Previous`, `Next`, `Last`) in `table_editor_view.rs`.
- [x] Extend non-regression tests in `crates/ui/src/components/mod.rs`.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Document findings and verification.
