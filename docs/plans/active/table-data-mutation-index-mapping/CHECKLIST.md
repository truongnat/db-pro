# Checklist — Table Data Mutation Failure Index Mapping

- [ ] Create plan documentation in `docs/plans/active/table-data-mutation-index-mapping/`.
- [ ] Implement index tracking and remapping in `TableDataService::apply_mutations_detailed`.
- [ ] Add unit test `apply_mutations_detailed_maps_statement_index_to_original_input_mutation` in `table_data_service.rs`.
- [ ] Run `cargo test -p db-pro-core`.
- [ ] Perform pre-commit checks.
- [ ] Submit PR.
