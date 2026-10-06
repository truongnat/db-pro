# Feature Plan: MySQL Batch Transaction Atomicity and Rollback

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), batch statements are currently executed directly against the connection pool (`sqlx::query(stmt).execute(&pool)`) without an enclosing transaction. If a statement in a batch fails, preceding statements remain committed, violating the `DbConnector::execute_batch` atomicity contract.

## Problem Statement & Severity
- **Severity**: P1 (Broken transaction semantics and non-atomic database mutations on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 134-148:
  ```rust
  async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
      let pool = self.get_pool(handle).await...;
      let mut total = 0;
      for stmt in statements {
          let result = sqlx::query(stmt).execute(&pool).await...;
          total += result.rows_affected();
      }
      Ok(total)
  }
  ```
- **Failure Scenario**: When running a batch of statements (e.g. schema changes in `SchemaService` or multi-statement DML mutations) on MySQL, if statement 1 succeeds and statement 2 fails (e.g. syntax error or foreign key violation), statement 1 is permanently committed to MySQL. The caller receives an error, but partial mutations are committed to the database.

## Scope & Implementation Plan
1. Create branch `fix/mysql-batch-transaction-atomicity` and plan directory `docs/plans/active/mysql-batch-transaction-atomicity/`.
2. Fix `MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs`.
   - Delegate `execute_batch` to `execute_transaction(handle, statements, &vec![false; statements.len()])` so that all batch statements execute atomically within a transaction and automatically rollback on failure.
   - Map `TransactionStatementResult::Affected` row counts to sum the total rows affected.
3. Add unit/integration regression tests in `crates/infrastructure/src/mysql/connector.rs`.
4. Run all relevant Rust test suites and quality gates (`cargo test -p db-pro-infrastructure`, `cargo check --workspace`, `cargo clippy --workspace --all-targets`).
5. Complete pre-commit steps.
6. Publish Pull Request against main.
