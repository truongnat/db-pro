# Findings: Postgres Parameterized Transaction Search Path

## Findings Summary

1. **Source Code Defect**:
   In `crates/infrastructure/src/postgres/connector.rs`, `execute_transaction` (lines 625-635) and `execute_batch` (lines 527-532) both invoke `super::search_path::set_local_search_path(&mut tx).await` to set `search_path` for the transaction to match `active_query_schema()`.
   However, `execute_parameterized_transaction` (lines 762-810) omitted both `set_local_search_path` and `track_backend`.

2. **Impact**:
   - `TableDataService::apply_mutations` calls `execute_parameterized_transaction`.
   - Edits to tables in non-public schemas lose active schema context in parameterized transactions.
   - Stop / Cancel cannot target active parameterized transactions because `pg_backend_pid` was not published to active backends.

3. **Remediation**:
   - Immediately after `pool.begin().await` in `execute_parameterized_transaction`:
     1. Track backend PID (`self.track_backend(handle.0, pid)`).
     2. Call `super::search_path::set_local_search_path(&mut tx).await`.
     3. If `set_local_search_path` returns `Err(error)`, roll back `tx` and return `TransactionFailurePhase::Begin` with `statement_index = 0` and `outcome = TransactionFailureOutcome::RolledBack`.
