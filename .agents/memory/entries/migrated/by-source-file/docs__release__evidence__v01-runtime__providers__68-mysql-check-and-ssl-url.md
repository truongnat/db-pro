---
schema_version: 1
id: "MEM-SRC-f0ec58ab34088f60"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/68-mysql-check-and-ssl-url.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/68-mysql-check-and-ssl-url.md"
source_sha256: "f0ec58ab34088f600f97b7ab3a9b8cd8a055cb4cde4fb1260d629ece2d39b192"
mirror_body_sha256: "f0ec58ab34088f600f97b7ab3a9b8cd8a055cb4cde4fb1260d629ece2d39b192"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# MySQL CHECK introspection + SSL URL mapping (#235 remainder)

## Claim

MySQL 8 CHECK constraints are read from `information_schema`, and the stored `SslMode` is
honoured on the sqlx connection URL instead of always negotiating `PREFERRED`.

## Changes

| Area | Detail |
|---|---|
| CHECK | `mysql/introspect.rs` joins `TABLE_CONSTRAINTS` + `CHECK_CONSTRAINTS` |
| TLS | `mysql_connection_url` appends `?ssl-mode=DISABLED\|REQUIRED\|VERIFY_CA\|VERIFY_IDENTITY` |
| Tests | `mysql_connection_url_maps_every_ssl_mode`; live `mysql_introspects_named_check_constraints` |

## Commands

```bash
cargo test -p db-pro-infrastructure mysql_connection_url_maps_every_ssl_mode --lib
# with MySQL DATABASE_URL:
cargo test -p db-pro-infrastructure --test mysql_integration \
  mysql_introspects_named_check_constraints -- --ignored
```

## Residual on #235

None for CHECK/SSL/change-set. Remaining if any: dedicated native UI/runtime MySQL harness beyond provider-path proofs (#234 conformance remainders may still apply).
