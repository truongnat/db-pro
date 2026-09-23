# Checklist — Table Data Mutation Failure Index Mapping

- [x] Reproduce & identify defect in `TableMutationExecution` statement index remapping and `validation_failure`
- [x] Gate statement index remapping on `failure.phase == TransactionFailurePhase::Statement`
- [x] Set `statement_index: 0` in `validation_failure`
- [x] Add unit tests in `table_data_service.rs` for `Validation`, `Begin`, and `Statement` failure index mapping
- [x] Add unit test `apply_mutations_detailed_maps_statement_index_complex_reordering` for `[Insert, Update, Delete, Insert]`
- [x] Run `cargo test -p db-pro-core` to verify all tests pass
- [x] Complete pre-commit checks (`cargo fmt`, `cargo check`, `cargo clippy`)
- [x] Publish Pull Request
