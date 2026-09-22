# Checklist — Table Data Mutation Failure Index Mapping

- [x] Reproduce & identify defect in `TableMutationExecution` statement index remapping and `validation_failure`
- [x] Gate statement index remapping on `failure.phase == TransactionFailurePhase::Statement`
- [x] Set `statement_index: 0` in `validation_failure`
- [x] Add unit tests in `table_data_service.rs` for `Validation`, `Begin`, and `Statement` failure index mapping
- [x] Run `cargo test -p db-pro-core` to verify all tests pass
- [x] Complete pre-commit checks (`cargo fmt`, `cargo check`, `cargo clippy`)
