# Checklist — Table Data Mutation Failure Index Mapping

- [x] Verify `apply_mutations_detailed` tracks original mutation indices.
- [x] Add unit test `apply_mutations_detailed_maps_statement_index_to_original_input_mutation`.
- [x] Add unit test `apply_mutations_detailed_maps_statement_index_complex_reordering`.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo check --workspace`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo test -p db-pro-core -p db-pro-infrastructure`.
- [x] Run `cargo build --release --locked -p db-pro-native`.
- [x] Run clean code scan.
- [x] Complete pre-commit steps.
- [x] Publish Pull Request.
