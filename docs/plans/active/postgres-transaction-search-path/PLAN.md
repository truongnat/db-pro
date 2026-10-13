# Feature Plan: PostgreSQL Parameterized Transaction Search Path Correctness

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `PostgresConnector::execute_parameterized_transaction` (`crates/infrastructure/src/postgres/connector.rs`), parameterized transactions started via `pool.begin().await` were missing the `set_local_search_path(&mut tx).await` call. Consequently, parameterized transactions (such as staged table mutations in `TableMutationExecution`) executed under PostgreSQL's default `search_path` rather than the active query schema configured for the connection (`active_query_schema()`), leading to relation not found errors or targeting the wrong schema.

## Severity
- **Severity**: P1 (Database mutation correctness, active schema search path bug, transaction execution correctness).
- **Failure Scenario**: A user configures an active query schema (e.g., `tenant_a`) on a PostgreSQL connection and saves staged table edits in the Table Data Editor. `TableMutationExecution` calls `execute_parameterized_transaction`. Because `set_local_search_path` was omitted, the transaction executes under `public` instead of `tenant_a`, failing with `relation "users" does not exist` or mutating the `public.users` table instead of `tenant_a.users`.

## Proposed Fix
1. In `PostgresConnector::execute_parameterized_transaction`, invoke `super::search_path::set_local_search_path(&mut tx).await` immediately after `pool.begin().await`.
2. If `set_local_search_path` returns an error, roll back the transaction and return a `TransactionFailure` with `phase: TransactionFailurePhase::Begin`, `statement_index: 0`, `outcome: TransactionFailureOutcome::RolledBack`, and the underlying `DbError`.
3. Add a regression test in `crates/infrastructure/tests/pg_integration.rs` verifying that parameterized transactions execute within the active schema `search_path`.
