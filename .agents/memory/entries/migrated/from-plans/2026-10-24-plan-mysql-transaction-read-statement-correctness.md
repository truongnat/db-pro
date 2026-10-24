---
schema_version: 1
id: "MEM-PLAN-mysql-transaction-read-statement-879904cdd744"
title: "Plan archive: Mysql Transaction Read Statement Correctness"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, mysql-transaction-read-statement-correct]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "879904cdd7442ac2820eb5af0ffe80305fba699c8cd6b7c3634065794f0168cb"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/mysql-transaction-read-statement-correctness/PLAN.md"
  - "docs/plans/active/mysql-transaction-read-statement-correctness/FINDINGS.md"
  - "docs/plans/active/mysql-transaction-read-statement-correctness/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/mysql-transaction-read-statement-correctness/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Feature Plan: MySQL Transaction Read-Statement Execution, Validation, and Batch Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector` (`crates/infrastructure/src/mysql/connector.rs`), batch execution (`execute_batch`) and multi-statement transactions (`execute_transaction`) had P1 transaction and mutation defects on MySQL:
1. `execute_batch` executed statements sequentially against a connection without an enclosing `START TRANSACTION` / `COMMIT` or `ROLLBACK`. On statement error or timeout, prior statements remained committed in MySQL, violating batch atomicity.
2. `execute_transaction` previously ignored `read_statements` and executed read statements as DML.

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, transaction atomicity & rollback defect on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 223–238 executed `conn.query_drop(statement)` directly in `execute_batch` without transaction boundaries or rollback handling.
- **Failure Scenario**: When running a batch execution with multiple statements on MySQL, if statement #2 fails or times out, statement #1 remains committed in MySQL (autocommit mode), causing partial state corruption and violating the `DbConnector::execute_batch` transaction contract.

## Scope & Implementation Plan
1. Refactor `MySqlConnector::execute_batch` to delegate execution to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
2. Sum `row_count` across returned `TransactionStatementResult::Affected` results.
3. Map `TransactionFailure` back to `DbError` on failure (preserving error messages and rollback semantics).
4. Add unit tests in `crates/infrastructure/src/mysql/connector.rs` verifying `execute_batch` delegation, error mapping, and affected row count summation.

## Provider Coverage
- **MySQL**: Directly fixed and verified.
- **PostgreSQL**: Implemented and verified in `PostgresConnector::execute_batch`.
- **SQLite**: Implemented and verified in `SqliteConnector::execute_batch`.
- **SQL Server**: Implemented and verified in `SqlServerConnector::execute_batch`.


## Source: `FINDINGS.md`

# Findings: MySQL Batch Execution Atomicity and Rollback

## Finding 1: MySQL `execute_batch` Lacked Transaction Atomicity
- **Path**: `crates/infrastructure/src/mysql/connector.rs`
- **Impact**: In `execute_batch`, batch SQL statements were executed sequentially on a pooled connection using `conn.query_drop(statement)` without an explicit `START TRANSACTION` / `COMMIT` or `ROLLBACK`.
- **Consequence**: If statement #2 in a batch failed or timed out, statement #1 remained committed in MySQL.
- **Resolution**: Refactor `MySqlConnector::execute_batch` to delegate execution to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`, ensuring statements execute inside an explicit transaction and roll back atomically if any statement fails or times out.


## Source: `VERIFICATION.md`

# Verification: MySQL Batch Execution Atomicity and Rollback

## Unit Testing
- Add unit tests in `crates/infrastructure/src/mysql/connector.rs` for `execute_batch`:
  1. `execute_batch` on an unknown handle returns `DbError::ConnectionFailed` (via `execute_transaction` Validation failure phase).
  2. Test `execute_batch` parameter delegation and affected-row count aggregation logic.

## Quality Gates
- Run `cargo fmt --all -- --check`
- Run `cargo check -p db-pro-core -p db-pro-infrastructure`
- Run `cargo clippy -p db-pro-core -p db-pro-infrastructure --all-targets -- -D warnings`
- Run `cargo test -p db-pro-core`

