---
schema_version: 1
id: "MEM-PLAN-postgres-parameterized-transacti-c144d434e6af"
title: "Plan archive: Postgres Parameterized Transaction Search Path"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, postgres-parameterized-transaction-searc]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "c144d434e6af54ed8f33690136cb298d2996d4496827b8bc567c96eefa9d0f49"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/postgres-parameterized-transaction-search-path/PLAN.md"
  - "docs/plans/active/postgres-parameterized-transaction-search-path/FINDINGS.md"
  - "docs/plans/active/postgres-parameterized-transaction-search-path/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/postgres-parameterized-transaction-search-path/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan: Fix missing search_path and backend tracking in PostgresConnector::execute_parameterized_transaction

## Core Directive & Evidence
In `PostgresConnector::execute_parameterized_transaction` (`crates/infrastructure/src/postgres/connector.rs`), parameterized transaction batches (used by `TableDataService::apply_mutations` for table row edits/deletes/inserts) begin a `sqlx::Transaction` but fail to invoke `super::search_path::set_local_search_path(&mut tx).await` and fail to track the active backend PID via `self.track_backend(handle.0, pid)`.

This violates the invariant that all database execution within a transaction respects the active query schema configured via `active_query_schema()`.

## Failure Scenario
1. A user connects to PostgreSQL with multiple schemas (e.g. `public` and `tenant_app`).
2. The user switches active query schema to `tenant_app`.
3. The user edits table rows in `TableDataService` (e.g. staging edits or inserts on table `users` in `tenant_app`).
4. `TableDataService::apply_mutations` calls `PostgresConnector::execute_parameterized_transaction`.
5. Because `set_local_search_path` is missing, the transaction connection defaults to PostgreSQL's default `search_path` (`$user, public`).
6. Unqualified or relative references in statements, triggers, sequence lookups, or custom types resolve against `public` instead of `tenant_app`, mutating the wrong table or failing with relation/type missing errors.

## Severity
**P1** - Database mutation correctness & transaction search_path defect in PostgreSQL.

## Scope
1. Update `PostgresConnector::execute_parameterized_transaction` in `crates/infrastructure/src/postgres/connector.rs`.
2. Add unit regression tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs`.
3. Add live integration test in `crates/infrastructure/tests/pg_integration.rs`.
4. Update `docs/plans/STATUS.md`.


## Source: `FINDINGS.md`

# Findings: Postgres Parameterized Transaction Search Path

## Findings Summary

1. **Source Code Defect**:
   In `crates/infrastructure/src/postgres/connector.rs`, `execute_transaction` (lines 625-635) and `execute_batch` (lines 527-532) both invoke `super::search_path::set_local_search_path(&mut tx).await` to set `search_path` for the transaction to match `active_query_schema()`.
   However, `execute_parameterized_transaction` (lines 762-810) omitted both `set_local_search_path` and `track_backend`.

2. **Impact**:
   - `TableDataService::apply_mutations` calls `execute_parameterized_transaction`.
   - Edits to tables in non-public schemas lose active schema context in parameterized transactions.
   - Stop / Cancel cannot target active parameterized transactions because `pg_backend_pid` was not published to active backends.

3. **Remediation**:
   - Immediately after `pool.begin().await` in `execute_parameterized_transaction`:
     1. Track backend PID (`self.track_backend(handle.0, pid)`).
     2. Call `super::search_path::set_local_search_path(&mut tx).await`.
     3. If `set_local_search_path` returns `Err(error)`, roll back `tx` and return `TransactionFailurePhase::Begin` with `statement_index = 0` and `outcome = TransactionFailureOutcome::RolledBack`.


## Source: `VERIFICATION.md`

# Verification: Postgres Parameterized Transaction Search Path Fix

## Verification Matrix

1. **Unit Test Verification**:
   - `crates/infrastructure/src/postgres/tests/connector_tests.rs`:
     - Test that `execute_parameterized_transaction` publishes `pg_backend_pid` before statement execution.
     - Test that `execute_parameterized_transaction` invokes `set_local_search_path`.

2. **Integration Test Verification**:
   - `crates/infrastructure/tests/pg_integration.rs`:
     - Test that `execute_parameterized_transaction` respects `active_query_schema()` during execution against live/fixture PostgreSQL.

3. **Workspace Quality Gates**:
   - `cargo check --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `python3 tools/cc-scan.py . --fail-on error`

