# Findings: Table Mutation Transaction Failure Index Bounds Safety

1. Pre-execution failures (`Validation`, `Begin`) and post-execution failures (`Commit`) report statement indices `0` or `statements.len()`.
2. Remapping `statement_index` to original mutation input index is only valid during `TransactionFailurePhase::Statement` when `statement_index < indexed_mutations.len()`.
3. `TableMutationExecution::execute` properly enforces both conditions, preventing out-of-bounds indexing or misattribution.
