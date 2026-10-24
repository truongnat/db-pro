---
schema_version: 1
id: "MEM-SRC-fd83b37cc1a72483"
title: "Source mirror: docs/plans/active/sqlserver-parameterized-transaction-semantics/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlserver-parameterized-transaction-semantics/FINDINGS.md"
source_sha256: "fd83b37cc1a724833ab01fa708f89fc45d0e176619a4626749e5b4241e899a9d"
mirror_body_sha256: "fd83b37cc1a724833ab01fa708f89fc45d0e176619a4626749e5b4241e899a9d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — SQL Server Parameterized Transaction Semantics

## Findings

1. `SqlServerConnector::execute_transaction` correctly implements `already_open` detection via `open_transaction_count(handle)`.
2. `SqlServerConnector::execute_parameterized_transaction` lacked `open_transaction_count(handle)` checks and unconditionally executed `BEGIN TRANSACTION`, `ROLLBACK TRANSACTION` on error, and `COMMIT TRANSACTION` on success.
3. On SQL Server, `ROLLBACK TRANSACTION` resets `@@TRANCOUNT` to 0 regardless of transaction nesting depth, so issuing `ROLLBACK TRANSACTION` when an outer transaction is active destroys the caller's active transaction and produces wrong error outcomes (`RolledBack` instead of `LeftOpen`).
