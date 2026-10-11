# Plan: Fix PostgreSQL Parameterized Transaction Search Path Isolation

## Problem Statement
`PostgresConnector::execute_parameterized_transaction` in `crates/infrastructure/src/postgres/connector.rs` fails to set the local `search_path` (`super::search_path::set_local_search_path(&mut tx)`). When executing parameterized transactions (such as table mutations in `TableDataService`), unqualified table or type references execute against the PostgreSQL default session search path (`"$user", public`) rather than the active schema configured in the session (`active_query_schema()`).

## Objectives
- Apply `super::search_path::set_local_search_path(&mut tx).await` in `PostgresConnector::execute_parameterized_transaction` immediately after `pool.begin().await`.
- Handle failure during `set_local_search_path` by rolling back `tx` and returning a `TransactionFailure` with `phase: TransactionFailurePhase::Begin`, `statement_index: 0`, and `outcome: TransactionFailureOutcome::RolledBack`.
- Add unit test coverage verifying `set_local_search_path` behavior inside transactions when `active_query_schema()` is set.

## Scope
- `crates/infrastructure/src/postgres/connector.rs`
- Unit tests in `crates/infrastructure/src/postgres/connector.rs`

## Target Branch
`fix/postgres-transaction-search-path`
