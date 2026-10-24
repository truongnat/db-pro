---
schema_version: 1
id: "MEM-PLAN-mysql-parameterized-transaction--bdd7a982bce3"
title: "Plan archive: Mysql Parameterized Transaction Connection Id"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, mysql-parameterized-transaction-connecti]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "bdd7a982bce3ebbf07a973fd04cb8b33b8740071267b1548c34371045eb037ec"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/mysql-parameterized-transaction-connection-id/PLAN.md"
  - "docs/plans/active/mysql-parameterized-transaction-connection-id/FINDINGS.md"
  - "docs/plans/active/mysql-parameterized-transaction-connection-id/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/mysql-parameterized-transaction-connection-id/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan: MySQL Parameterized Transaction Active Connection Tracking

## State
`IMPLEMENTING`

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, query cancellation & provider defect on MySQL).
- **Problem**: In `MySqlConnector::execute_parameterized_transaction` (`crates/infrastructure/src/mysql/connector.rs`), parameterized transaction batches begin a `sqlx::Transaction` but fail to query `SELECT CONNECTION_ID()` and hold a `track_guard` (`ActiveMySql`) guard during statement execution. As a result, when a user cancels a running parameterized table mutation (such as a large batch row update or delete), `MySqlConnector::cancel` finds an empty `active_connections` list and fails to issue `KILL QUERY <connection_id>`, leaving the MySQL server executing the mutation un-cancelable from DB Pro.
- **Evidence**: `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs` lacks the `SELECT CONNECTION_ID()` tracking call present in `MySqlConnector::query`, `MySqlConnector::execute`, `MySqlConnector::execute_transaction`, and `PostgresConnector::execute_parameterized_transaction`.

## Failure Scenario
1. A user connects to a MySQL database and opens a table in the Table Data Editor.
2. The user stages mutations affecting many rows (e.g., updating or deleting thousands of rows in a table).
3. `TableDataService::apply_mutations` invokes `MySqlConnector::execute_parameterized_transaction`.
4. While the transaction is running on MySQL, the user clicks "Stop" / Cancel in DB Pro.
5. `MySqlConnector::cancel` reads `entry.active_connections`, which is empty because `execute_parameterized_transaction` never registered its connection ID.
6. The cancellation request silently succeeds without executing `KILL QUERY <connection_id>`, and the MySQL server continues processing the transaction unhindered until completion.

## Scope & Solution
1. Update `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs`:
   - Query `SELECT CONNECTION_ID()` on `&mut tx` after `pool.begin().await`.
   - Acquire and hold `_active = match sqlx::query_scalar::<_, u64>("SELECT CONNECTION_ID()").fetch_one(&mut *tx).await { Ok(id) => Some(self.track_guard(handle.0, id)), Err(_) => None }`.
2. Add regression unit test in `crates/infrastructure/src/mysql/tests/connector_tests.rs`.
3. Update `docs/plans/STATUS.md`.


## Source: `FINDINGS.md`

# Findings: MySQL Parameterized Transaction Active Connection Tracking

## Root Cause
In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_parameterized_transaction` creates a transaction via `pool.begin().await` but does not query `SELECT CONNECTION_ID()` to populate `active_connections`.

## Impact
Query cancellation calls (`cancel(&ConnectionHandle)`) rely on `active_connections` containing the server connection thread ID to issue `KILL QUERY <connection_id>`. Without connection tracking during parameterized transactions, table mutations (staged edits, inserts, deletes) cannot be canceled by the user if they hang or take a long time to complete.


## Source: `VERIFICATION.md`

# Verification: MySQL Parameterized Transaction Active Connection Tracking

## Automated Tests
- Unit test `execute_parameterized_transaction_publishes_connection_id_before_statements` added to `crates/infrastructure/src/mysql/tests/connector_tests.rs`.
- `cargo check -p db-pro-infrastructure` passes.
- `cargo test -p db-pro-infrastructure` passes.

## Quality Gates
- `cargo fmt`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`

