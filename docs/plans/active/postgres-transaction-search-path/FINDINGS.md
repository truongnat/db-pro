# Findings: PostgreSQL Parameterized Transaction Search Path Correctness

## Root Cause Analysis
In `PostgresConnector::execute_parameterized_transaction` (`crates/infrastructure/src/postgres/connector.rs`), a transaction is acquired via `pool.begin().await`.
Unlike `execute_batch` and `execute_transaction`, `execute_parameterized_transaction` omitted calling `super::search_path::set_local_search_path(&mut tx).await`.

`set_local_search_path` reads the thread-local active query schema from `active_query_schema()` and issues:
`SELECT set_config('search_path', quote_ident($1), true)`
Setting `is_local = true` applies `search_path` to the active transaction block.

Without this call, parameterized transactions operated under the default PostgreSQL search path (`"$user", public`), bypassing the user's active query schema configuration.

## Impact Analysis
- Table data mutations (INSERT / UPDATE / DELETE) submitted via `TableMutationExecution` fail when operating on non-public schemas.
- If identical table names exist in `public`, mutations could affect rows in `public` instead of the target active schema.

## Solution
Invoke `super::search_path::set_local_search_path(&mut tx).await` in `execute_parameterized_transaction` directly after transaction start.
If setting the search path fails, perform rollback and return `TransactionFailurePhase::Begin` with `statement_index = 0`.
