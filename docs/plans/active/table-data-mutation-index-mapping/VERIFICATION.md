# Verification — Table Data Mutation Failure Index Mapping

## Unit Test Coverage

Added unit tests in `crates/core/src/application/table_data_service.rs`:

1. `apply_mutations_detailed_maps_statement_index_to_original_input_mutation`:
   - Verifies that when execution fails on reordered statement at index 1 (`Insert`), `statement_index` is mapped back to original input index 0 (`Insert`).

2. `apply_mutations_detailed_maps_statement_index_complex_reordering`:
   - Verifies `[Insert, Update, Delete, Insert]` reordering. Execution order is `[Delete, Update, Insert, Insert]`. A failure at exec index 0 (`Delete`) maps back to original input index 2.

3. `apply_mutations_detailed_retains_zero_statement_index_on_begin_failure`:
   - Verifies that when `execute_parameterized_transaction` fails with `phase: Begin`, `statement_index` remains `0` and is NOT remapped to `indexed_mutations[0].0`.

4. `apply_mutations_detailed_retains_zero_statement_index_on_validation_failure`:
   - Verifies that when `apply_mutations_detailed` fails with `phase: Validation` (e.g. read-only connection), `statement_index` is `0` (not `self.mutations.len()`).

## Automated Execution

Command: `cargo test -p db-pro-core`
Result: pending re-run after merging `origin/main` into this branch.
