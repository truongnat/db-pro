---
schema_version: 1
id: "MEM-SRC-011842978d91473f"
title: "Source mirror: docs/plans/active/sqlserver-parameterized-transaction-semantics/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlserver-parameterized-transaction-semantics/PLAN.md"
source_sha256: "011842978d91473f4cb548cdd38051d5f88f4c4dfdb9b8797085d9659a376be3"
mirror_body_sha256: "011842978d91473f4cb548cdd38051d5f88f4c4dfdb9b8797085d9659a376be3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
