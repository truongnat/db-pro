---
schema_version: 1
id: "MEM-SRC-ec727ba78d14d407"
title: "Source mirror: docs/plans/active/mysql-parameterized-transaction-connection-id/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/mysql-parameterized-transaction-connection-id/FINDINGS.md"
source_sha256: "ec727ba78d14d40705c3e231f31bbb00be5fe77d5df2b029a10a2656144ec1bf"
mirror_body_sha256: "ec727ba78d14d40705c3e231f31bbb00be5fe77d5df2b029a10a2656144ec1bf"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings: MySQL Parameterized Transaction Active Connection Tracking

## Root Cause
In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_parameterized_transaction` creates a transaction via `pool.begin().await` but does not query `SELECT CONNECTION_ID()` to populate `active_connections`.

## Impact
Query cancellation calls (`cancel(&ConnectionHandle)`) rely on `active_connections` containing the server connection thread ID to issue `KILL QUERY <connection_id>`. Without connection tracking during parameterized transactions, table mutations (staged edits, inserts, deletes) cannot be canceled by the user if they hang or take a long time to complete.
