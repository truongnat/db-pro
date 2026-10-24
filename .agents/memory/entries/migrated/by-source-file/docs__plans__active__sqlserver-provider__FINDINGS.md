---
schema_version: 1
id: "MEM-SRC-f90214ab1f64ea39"
title: "Source mirror: docs/plans/active/sqlserver-provider/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlserver-provider/FINDINGS.md"
source_sha256: "f90214ab1f64ea39186b64b6c38fe605a13a82fe3701599ea902f5e4dcfe9628"
mirror_body_sha256: "f90214ab1f64ea39186b64b6c38fe605a13a82fe3701599ea902f5e4dcfe9628"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# SQL Server provider findings (#260)

## Baseline findings

| Severity | Finding | Evidence | Disposition |
|---|---|---|---|
| P1 | There is no SQL Server provider arm in the core driver enum or composite connector. | `cc399501338b7f38342a1bb12b9990e048537312`; `crates/core/src/domain/connection.rs`, `crates/infrastructure/src/connector.rs` | Resolved in `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`. |
| P1 | The workspace has no TDS driver dependency, so a UI-only or enum-only change would be non-functional. | `cc399501338b7f38342a1bb12b9990e048537312`; root `Cargo.toml` | Resolved in `774a4e7f6ce8d88939988711e8fadf0588f1c3c1` with Tiberius/Tokio compat. |
| P1 | Issue acceptance requires live SQL Server evidence, but no SQL Server fixture is currently running in the environment. | GitHub #260 implementation-audit comments; `docker ps` on 2026-09-17 | Open; fixture lane required. |
| P2 | SQL Server-specific administration and PostgreSQL-only features need explicit capability reasons rather than generic errors. | `cc399501338b7f38342a1bb12b9990e048537312`; `crates/core/src/domain/capabilities.rs` | Resolved in `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`; automated capability tests pass. |

## Review findings

No independent review has run yet. Self-review at `774a4e7f6ce8d88939988711e8fadf0588f1c3c1` found no P0/P1 regression; live fixture and native UI evidence remain blocked.
