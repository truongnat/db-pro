# Findings: PostgreSQL Parameterized Transaction Search Path Isolation

## Finding 1: `PostgresConnector::execute_parameterized_transaction` Omitted `set_local_search_path`
- **Location**: `crates/infrastructure/src/postgres/connector.rs`
- **Impact**: In `execute_parameterized_transaction`, `super::search_path::set_local_search_path(&mut tx)` was not called after starting a transaction via `pool.begin().await`. In contrast, both `execute_batch` and `execute_transaction` invoke `set_local_search_path(&mut tx)`.
- **Consequence**: Staged table mutations executed via `TableDataService` (which call `execute_parameterized_transaction`) ran with PostgreSQL's default session `search_path` instead of the active query schema set by `active_query_schema()`. Unqualified table or custom type references failed or mutated the wrong schema.
- **Resolution**: Call `super::search_path::set_local_search_path(&mut tx).await` in `execute_parameterized_transaction` immediately after `pool.begin().await`. On failure, roll back `tx` and return `TransactionFailure` with `phase: TransactionFailurePhase::Begin`, `statement_index: 0`, and `outcome: TransactionFailureOutcome::RolledBack`.
