---
schema_version: 1
id: "MEM-SRC-e0ceffefeb2733f0"
title: "Source mirror: docs/plans/active/provider-correct-generated-sql/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/provider-correct-generated-sql/CHECKLIST.md"
source_sha256: "e0ceffefeb2733f0235f760334e9bab38e2be5a0482bd5881f70901df9129720"
mirror_body_sha256: "e0ceffefeb2733f0235f760334e9bab38e2be5a0482bd5881f70901df9129720"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #441 / #442

Baseline for source findings: `76b120724a7542de8b587a002811ddacad28156a`.
State: REVIEW; worker implementation complete, parent execution/closure pending.

## Implementation acceptance

- [x] Inspect shared dialect, all private DDL helpers and public service consumer, adjacent tests and reload validation contract.
- [x] Reuse existing dialect pagination/order hooks for PK reload without changing SqlDialect ports, binding order or the at-most-one-row cap.
- [x] Add regression cases for composite reload SQL and actual parameter values on PostgreSQL/SQLite/MySQL/SQL Server; reject missing/mismatched keys.
- [x] Reuse DriverDialect quoting for reconstructed table/schema/column/PK/CHECK/FK/index identifiers; delete old quote helper/test import.
- [x] Preserve SQLite unqualified naming/inline unique constraints and PostgreSQL authoritative index SQL/ALTER TABLE FKs.
- [x] Make reconstructed index names unqualified and retain inline MySQL/SQL Server FKs with composite order/actions.
- [x] Add public SchemaService output matrix covering reserved words and all quote delimiters, and PostgreSQL native index/deferred composite FK preservation.
- [x] Keep trigger-specific behavior and expression bodies unchanged.
- [x] Provide independent public-service live probe, separate baseline reproduction cases and honest provider prerequisites.
- [x] Address independent-review producer contracts: SQL Server catalog action normalization, MySQL cross-database FK targets and scoped catalog joins.
- [x] Preserve MySQL supporting/unique indexes inline before FK creation, with no duplicate post-create index emission.
- [x] Add deterministic catalog-action unit coverage and ignored real-provider SchemaService round-trips (SQL Server default/SET actions; MySQL cross-database key enforcement and duplicate constraint names).

## Integration-owner gates (not executed by worker)

- [x] Core regression suites executed in workspace tests; final integrated count 2,002 passed / 0 failed / 48 ignored before the later separate #533 null-type correction.
- [x] Formatting/check/tests executed; workspace Clippy still fails on unchanged app diagnostics, not claimed green.
- [x] Independent public `pk` and `ddl` probes PASS on real MySQL 8.4.11 (ANSI_QUOTES absent), SQLite and PostgreSQL 18; MySQL cross-database FK regression explicitly executed (1/0/0).
- [x] Independent re-review ACCEPT after catalog action/schema/index corrections.
- [ ] SQLServer live PK/DDL/action tests: NOT VERIFIED; #441 stays OPEN.
- [ ] Publish candidate and close only #442; attach exact SHA/evidence to still-open #441.

Unchecked gates are not success claims; leave issues open until parent verifies their required behavior.
