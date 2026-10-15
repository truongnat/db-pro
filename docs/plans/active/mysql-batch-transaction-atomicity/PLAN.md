# Feature Plan: MySQL Batch Transaction Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), batch statements were previously executed in a loop directly against the connection pool (`&pool`) without starting a transaction.
If an intermediate statement in a batch failed, earlier statements were left permanently committed in MySQL, violating atomicity and the `DbConnector::execute_batch` contract.

## Problem & Severity
- **Severity**: P1 (Broken transaction semantics and database mutation correctness failure on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs`:
  Executing batch statements against `&pool` in a loop without `pool.begin().await`.
- **Failure Scenario**: When running a batch execution (such as multi-statement grid updates or table batch mutations) where statement 1 succeeds and statement 2 fails, statement 1 remains committed on MySQL instead of rolling back atomically.

## Scope & Implementation Plan
1. Delegate `MySqlConnector::execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
2. Sum the row counts of returned `TransactionStatementResult::Affected` values, returning `failure.error` on transaction failure.
3. Add regression unit tests in `crates/infrastructure/src/mysql/connector.rs`.
