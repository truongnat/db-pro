# Findings: PostgreSQL Parameterized Transaction Search Path Isolation

## Discovery
In `PostgresConnector` (`crates/infrastructure/src/postgres/connector.rs`), `execute_batch` and `execute_transaction` both call `super::search_path::set_local_search_path(&mut tx).await` right after `pool.begin().await`. However, `execute_parameterized_transaction` omitted this call.

## Severity
**P1** - Database mutation correctness & provider-specific schema isolation failure.

## Root Cause
`execute_parameterized_transaction` created a transaction via `pool.begin().await` and directly proceeded to statement execution without setting the active schema search path. When `active_query_schema()` was set to a non-public schema, table mutations issued via `TableDataService` / `TableMutationExecution` executed under the default session search path (`public` or `$user`), causing writes to target wrong tables or fail with table not found errors.
