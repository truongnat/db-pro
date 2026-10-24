---
schema_version: 1
id: "MEM-SRC-4561e8a4ee65d084"
title: "Source mirror: docs/plans/completed/sqlserver-transaction-begin-index/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/sqlserver-transaction-begin-index/CHECKLIST.md"
source_sha256: "4561e8a4ee65d084f6630151f5299bba41b85a463eae9333cb64832d92e3f631"
mirror_body_sha256: "4561e8a4ee65d084f6630151f5299bba41b85a463eae9333cb64832d92e3f631"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — SQL Server Transaction Begin Index Fix

- [x] Update `execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Validation` phase.
- [x] Update `execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Begin` phase.
- [x] Update `execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Begin` phase.
- [x] Add unit test `execute_transaction_reports_zero_statement_index_on_validation_failure`.
- [x] Add unit test `execute_transaction_reports_zero_statement_index_on_begin_failure`.
- [x] Add unit test `execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`.
- [x] Verify unit tests (`cargo test -p db-pro-infrastructure --lib statement_index` — 3 passed).
- [x] Pass all quality gates.

Implemented in commit `61e97658`; verified on HEAD `aaa7a69e` (2026-09-23).
