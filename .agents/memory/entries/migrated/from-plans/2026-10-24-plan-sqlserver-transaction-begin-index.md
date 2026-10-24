---
schema_version: 1
id: "MEM-PLAN-sqlserver-transaction-begin-inde-4a12b385405e"
title: "Plan archive: Sqlserver Transaction Begin Index"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sqlserver-transaction-begin-index]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "4a12b385405e626b68bf436f64bde6bf9a33464cfa1aa88be3fca778263e23a6"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/sqlserver-transaction-begin-index/PLAN.md"
  - "docs/plans/completed/sqlserver-transaction-begin-index/FINDINGS.md"
  - "docs/plans/completed/sqlserver-transaction-begin-index/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/sqlserver-transaction-begin-index/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — Fix SQL Server Transaction Begin and Validation Failure Statement Index

## Baseline

- State: COMPLETED (implemented in `61e97658`, verified on HEAD `aaa7a69e`, 2026-09-23)
- Target branch: `fix/sqlserver-transaction-begin-index`
- Severity: P1 / P2 (Transaction failure reporting contract mismatch)
- Canonical Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Objective

Fix `SqlServerConnector::execute_transaction` and `SqlServerConnector::execute_parameterized_transaction` so that when a transaction fails during `TransactionFailurePhase::Validation` or `TransactionFailurePhase::Begin`, `TransactionFailure::statement_index` is reported as `0` (indicating 0 statements were executed) rather than `statements.len()` (out-of-bounds).

## Scope

1. Update `SqlServerConnector::execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Validation` and `Begin` failure phases.
2. Update `SqlServerConnector::execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for the `Begin` failure phase.
3. Add unit tests in `crates/infrastructure/src/sqlserver/connector.rs` verifying that `statement_index` is `0` when `Validation` or `Begin` fails.
4. Run core unit tests (`cargo test -p db-pro-core`).


## Source: `FINDINGS.md`

# Findings — SQL Server Transaction Failure Statement Index Defect

## Description

In `crates/infrastructure/src/sqlserver/connector.rs`, the `SqlServerConnector` implementation of `execute_transaction` and `execute_parameterized_transaction` contained a defect in failure reporting during the `Validation` and `Begin` phases.

When metadata validation failed or when `BEGIN TRANSACTION` failed (for instance due to an invalid handle or connection state), `SqlServerConnector` constructed a `TransactionFailure` with `statement_index` set to `statements.len()`.

## Impact

1. **Out-of-bounds Statement Index**: For a transaction batch with $N$ statements ($N > 0$), if `BEGIN TRANSACTION` fails before any statements are executed, reporting `statement_index = N` indicates that $N$ statements were processed prior to failure.
2. **Caller Mapping Mismatch**: Caller routines such as `TableDataService::apply_mutations_detailed` attempt to map `failure.statement_index` back to original staged mutation indices. When `statement_index == N` (where $N = \text{indexed\_mutations.len()}$), the boundary check `if failure.statement_index < indexed_mutations.len()` fails, leaving the out-of-bounds index unmapped and misattributing the failure.
3. **Provider Inconsistency**: SQLite, PostgreSQL, and MySQL connectors all report `statement_index: 0` for `Validation` and `Begin` failure phases.

## Fix Strategy

Change `statement_index: statements.len()` to `statement_index: 0` for both `TransactionFailurePhase::Validation` and `TransactionFailurePhase::Begin` in `SqlServerConnector`.


## Source: `VERIFICATION.md`

# Verification — SQL Server Transaction Begin Index Fix

State: COMPLETED
Implementation commit: `61e97658` ("fix(sqlserver): report statement_index 0 on transaction begin and validation failures")
Verified on: 2026-09-23 (HEAD `aaa7a69e`)

## Automated Tests Executed

- `cargo test -p db-pro-infrastructure --lib statement_index` — **3 passed / 0 failed**:
  - `sqlserver::connector::tests::execute_transaction_reports_zero_statement_index_on_validation_failure`
  - `sqlserver::connector::tests::execute_transaction_reports_zero_statement_index_on_begin_failure`
  - `sqlserver::connector::tests::execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`

## Source evidence

`crates/infrastructure/src/sqlserver/connector.rs`:
- `execute_transaction` — `TransactionFailurePhase::Validation` and `TransactionFailurePhase::Begin` now set `statement_index: 0` (were `statements.len()`).
- `execute_parameterized_transaction` — `TransactionFailurePhase::Begin` sets `statement_index: 0`.

This matches the SQLite/PostgreSQL/MySQL connectors, so the caller boundary check
`if failure.statement_index < indexed_mutations.len()` in `TableDataService::apply_mutations_detailed`
maps the failure correctly instead of leaving an out-of-bounds index unmapped.

## Provider matrix

| Provider | Supported operation | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| SQL Server | yes | PASS (unit) | inherited by parent `sqlserver-provider` (no live SQL Server fixture here) | n/a |
| PostgreSQL / SQLite / MySQL | yes (already correct) | PASS | n/a for this fix | n/a |

The Begin/Validation `statement_index` invariant is provider-logic that unit tests fully exercise
(no statements run before the failure, so no live server is required to prove index `0`). Live SQL
Server runtime for the broader provider remains tracked under `sqlserver-provider`.

## Findings

- P0: 0
- P1: 0 (the reported statement-index defect is fixed and regression-tested)
- P2: 0

## Verification Status

COMPLETED — code merged to `main` (`61e97658`) and regression tests pass.

