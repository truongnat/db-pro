# Findings: MySQL Batch Transaction Atomicity

## Problem Statement
`MySqlConnector::execute_batch` executed statements directly on `&pool` in a loop without starting a transaction.

## Evidence
`crates/infrastructure/src/mysql/connector.rs` previously ran `sqlx::query(stmt).execute(&pool)` directly inside a loop.

## Impact & Severity
Severity: P1.
In MySQL, DML operations in auto-commit mode commit immediately upon execution. A multi-statement batch with a failure in statement N leaves statements 1..(N-1) committed, causing data corruption and violating the `DbConnector::execute_batch` contract.
