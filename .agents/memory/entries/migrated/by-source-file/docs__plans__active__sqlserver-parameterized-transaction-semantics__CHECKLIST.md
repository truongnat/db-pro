---
schema_version: 1
id: "MEM-SRC-2f2189651d8dd844"
title: "Source mirror: docs/plans/active/sqlserver-parameterized-transaction-semantics/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlserver-parameterized-transaction-semantics/CHECKLIST.md"
source_sha256: "2f2189651d8dd8446dff890cd24b8fb3bcee592846a94a8ed65b0308869f9221"
mirror_body_sha256: "2f2189651d8dd8446dff890cd24b8fb3bcee592846a94a8ed65b0308869f9221"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — SQL Server Parameterized Transaction Semantics

- [x] Check `open_transaction_count(handle)` in `SqlServerConnector::execute_parameterized_transaction`
- [x] Skip `BEGIN TRANSACTION` and `COMMIT TRANSACTION` when `already_open` is true
- [x] Return `TransactionFailureOutcome::LeftOpen` without `ROLLBACK TRANSACTION` on error when `already_open` is true
- [x] Add unit test for `execute_parameterized_transaction` on unconnected handle
- [x] Run `cargo check --workspace`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo test --workspace`
- [x] Run clean code audit `python3 tools/cc-scan.py . --fail-on error`
