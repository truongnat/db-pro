---
schema_version: 1
id: "MEM-SRC-9e4d4e90610342ae"
title: "Source mirror: docs/plans/completed/sqlserver-transaction-begin-index/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/sqlserver-transaction-begin-index/PLAN.md"
source_sha256: "9e4d4e90610342aec2b2b560648992fca841cbc09259206dc75baef84674ad0f"
mirror_body_sha256: "9e4d4e90610342aec2b2b560648992fca841cbc09259206dc75baef84674ad0f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
