---
schema_version: 1
id: "MEM-SRC-3c27167f1cb2d998"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/VERIFICATION.md"
source_sha256: "3c27167f1cb2d998f2a7828738722fe27383864465d9f8576461a3c3cfcbee19"
mirror_body_sha256: "3c27167f1cb2d998f2a7828738722fe27383864465d9f8576461a3c3cfcbee19"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification

Baseline bceb735727f81180dad678defee1000bc547d97b. Initial144 open; selected #499/#508/#452/#486 have no matching open implementation PR. No test/provider pass claimed yet; append actual commands/outcomes, distinguishing baseline and candidate.

## Parent baseline execution

Exact source export: bceb735727f81180dad678defee1000bc547d97b (no worktree). Identical standalone Rust public-API probe projects use only path dependencies switched between export/main; `cargo run --offline --manifest-path /tmp/db-pro-wave140-probes/baseline/Cargo.toml -- <case>`.

| Case | Observed baseline result | Exit |
|---|---|---|
| mysql / #452 | ConnectionFailed: MySQL connect failed: error with configuration: empty host; fixture credential delimiters were parsed as URL structure | 1 |
| event / #499 | Confirmed public service create fails QuerySyntaxAt: syntax error near semicolon, position122 | 1 |
| deps / #508 | Public connector Internal: column notation .relnamespace applied to type regclass, not a composite type | 1 |
| csv / #486 | Public preview/stream/SQLite import PASS8 cases, exact header/name mapping and data semantics | 0 |

The first throwaway compilation used SqliteConnector instead of actual SQLiteConnector (E0432); corrected both probes before execution. This harness typo is not a product RED result. The dependency oracle queries the provider's native identity for the known enum type rather than pinning its numeric OID/display spelling.

## Fixture evidence

- PostgreSQL18.6, loopback25432, disposable dbpro_smoke; current_user dbpro, default search_path `$user, public`. No existing application service used.
- MySQL8.4.11, loopback25433. Initial attempted `--skip-ssl` fixture exited because MySQL8.4 removed that flag; corrected via documented `--tls-version=`. Server tls_version and session Ssl_cipher observed empty. Source: https://dev.mysql.com/doc/refman/8.4/en/encrypted-connection-protocols-ciphers.html .
- Provisioned local-only reserved/Unicode fixture user `u@:#/?%Việt`, database `db#@:?%Việt`, account host `%`; no credential URL is used for candidate tests. Passwords are disposable fixture-only, not production secrets.
- Adjacent BOM import consumer test ran on unchanged production code:1 passed/0 failed/0 ignored, exit0. No duplicate BOM implementation added.
