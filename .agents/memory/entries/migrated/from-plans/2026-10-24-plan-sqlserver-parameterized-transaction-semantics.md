---
schema_version: 1
id: "MEM-PLAN-sqlserver-parameterized-transact-447eb08a30f4"
title: "Plan archive: Sqlserver Parameterized Transaction Semantics"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sqlserver-parameterized-transaction-sema]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "447eb08a30f4d3ea01fe3ab8efbd7d682e3a4fc99ebc92228d324c864823c791"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sqlserver-parameterized-transaction-semantics/PLAN.md"
  - "docs/plans/active/sqlserver-parameterized-transaction-semantics/FINDINGS.md"
  - "docs/plans/active/sqlserver-parameterized-transaction-semantics/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sqlserver-parameterized-transaction-semantics/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# SQL Server Parameterized Transaction Semantics

## State

`IMPLEMENTING`

## Goal

Ensure `SqlServerConnector::execute_parameterized_transaction` respects existing open transactions on a connection handle, avoiding improper `BEGIN TRANSACTION`, `ROLLBACK TRANSACTION`, or `COMMIT TRANSACTION` control calls that destroy caller transactions or report wrong failure outcomes.

## Problem Statement

In `SqlServerConnector::execute_parameterized_transaction`, when a transaction is already active on a connection (`already_open`), the connector unconditionally issues `BEGIN TRANSACTION`, `ROLLBACK TRANSACTION` (on statement or invariant failure), and `COMMIT TRANSACTION` (on success). In SQL Server, `ROLLBACK TRANSACTION` rolls back all open transaction levels (`@@TRANCOUNT` drops to 0), destroying the caller's outer transaction and falsely reporting `TransactionFailureOutcome::RolledBack` instead of `TransactionFailureOutcome::LeftOpen`. When statements succeed, issuing `COMMIT TRANSACTION` decrements `@@TRANCOUNT` prematurely on the active transaction.

## Evidence & Failure Scenario

- **Code path**: `SqlServerConnector::execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs`.
- **Comparison**: `SqlServerConnector::execute_transaction` correctly checks `already_open = self.open_transaction_count(handle) > 0`, skips `BEGIN`/`COMMIT`, and returns `TransactionFailureOutcome::LeftOpen` without executing `ROLLBACK` when `already_open`. `execute_parameterized_transaction` lacks this logic.
- **Failure Scenario**:
  1. An interactive transaction session or query tab opens a transaction on a SQL Server connection handle (`@@TRANCOUNT = 1`).
  2. A staged table mutation calls `execute_parameterized_transaction`.
  3. `execute_parameterized_transaction` issues `BEGIN TRANSACTION` (`@@TRANCOUNT` becomes 2).
  4. A statement fails or yields 0 affected rows (`expect_affected_rows` check fails).
  5. `execute_parameterized_transaction` calls `self.rollback(handle)`, executing `ROLLBACK TRANSACTION`, which resets `@@TRANCOUNT` to 0, wiping out the caller's uncommitted outer transaction and reporting `TransactionFailureOutcome::RolledBack` instead of `LeftOpen`.
- **Severity**: P1 (database mutation correctness & transaction/rollback defect).

## Scope

1. Update `SqlServerConnector::execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs`:
   - Inspect `self.open_transaction_count(handle)` prior to beginning the transaction.
   - If `already_open`, skip executing `BEGIN TRANSACTION` and `COMMIT TRANSACTION`.
   - On statement or invariant failure, if `already_open`, return `TransactionFailureOutcome::LeftOpen` without issuing `ROLLBACK TRANSACTION`.
2. Add regression unit tests in `crates/infrastructure/src/sqlserver/tests/connector_tests.rs`.


## Source: `FINDINGS.md`

# Findings — SQL Server Parameterized Transaction Semantics

## Findings

1. `SqlServerConnector::execute_transaction` correctly implements `already_open` detection via `open_transaction_count(handle)`.
2. `SqlServerConnector::execute_parameterized_transaction` lacked `open_transaction_count(handle)` checks and unconditionally executed `BEGIN TRANSACTION`, `ROLLBACK TRANSACTION` on error, and `COMMIT TRANSACTION` on success.
3. On SQL Server, `ROLLBACK TRANSACTION` resets `@@TRANCOUNT` to 0 regardless of transaction nesting depth, so issuing `ROLLBACK TRANSACTION` when an outer transaction is active destroys the caller's active transaction and produces wrong error outcomes (`RolledBack` instead of `LeftOpen`).


## Source: `VERIFICATION.md`

# Verification — SQL Server Parameterized Transaction Semantics

## Quality Gates

- `cargo check --workspace` (PASS)
- `cargo clippy --workspace --all-targets -- -D warnings` (PASS)
- `cargo test --workspace` (PASS, 665 passed)
- `python3 tools/cc-scan.py . --fail-on error` (PASS)

## Verification Evidence

- Automated unit tests in `crates/infrastructure/src/sqlserver/tests/connector_tests.rs`:
  - `execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`
  - `execute_parameterized_transaction_empty_statements_and_unconnected_handle`
  verifying that `execute_parameterized_transaction` queries `@@TRANCOUNT` during the initial `Begin` phase and correctly returns `TransactionFailurePhase::Begin` with `statement_index = 0` and `TransactionFailureOutcome::NotStarted` on failure.

