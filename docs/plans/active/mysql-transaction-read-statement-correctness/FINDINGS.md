# Findings: MySQL Batch Execution Atomicity and Pre-execution Failure Indexing

## Finding 1: MySQL `execute_batch` Lacked Transaction Atomicity
- **Path**: `crates/infrastructure/src/mysql/connector.rs`
- **Impact**: In `execute_batch`, batch SQL statements were executed sequentially on a pooled connection without an explicit `START TRANSACTION` / `COMMIT` or `ROLLBACK`.
- **Consequence**: If statement #2 in a batch failed or timed out, statement #1 remained committed in MySQL.
- **Resolution**: Refactor `MySqlConnector::execute_batch` to delegate execution to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`, ensuring statements execute inside an explicit transaction and roll back atomically if any statement fails or times out.

## Finding 2: `execute_parameterized_transaction` Pre-execution Failure Indexing
- **Path**: `crates/infrastructure/src/mysql/connector.rs`
- **Impact**: `execute_parameterized_transaction` pre-execution failure phases (`Validation` and `Begin`) must report `statement_index = 0`.
- **Consequence**: Lack of explicit unit test coverage left `execute_parameterized_transaction` vulnerable to regressions where pre-execution failures report non-zero indices, breaking downstream caller remapping logic (e.g. `TableMutationExecution`).
- **Resolution**: Added `execute_parameterized_transaction_reports_validation_failure_on_unknown_handle` unit test to verify `statement_index == 0` on `Validation` failure phase.
