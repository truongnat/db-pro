---
schema_version: 1
id: "MEM-SRC-9c7f83cdd0e4e180"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/64-mysql-binder-dialect-ui.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/64-mysql-binder-dialect-ui.md"
source_sha256: "9c7f83cdd0e4e1806a9bc3318fb5d4eb53ee13d488f703d0b501bf6de8e95655"
mirror_body_sha256: "9c7f83cdd0e4e1806a9bc3318fb5d4eb53ee13d488f703d0b501bf6de8e95655"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# MySQL binder, dialect, and UI surface (#235 / #234)

- Session: 2026-09-15
- Issues: **#235** (MySQL provider), **#234** (provider SDK remainders 2/5)
- Tree: lands on `main` after this evidence file's commit

## What landed

| Change | Why |
|---|---|
| `mysql/query_mapper::bind_params` + `query`/`execute` use `query_with` | write half of criterion 3 — parameters are bound, not dropped |
| `CompositeConnector::dialect` returns `MySqlDialect` | unblocks table-data dialect consumers; removes the hard error |
| `DatabaseCapabilities::mysql()` advertises `parameters` + `positional_parameters` | capability truthfulness — flags match shipping code paths |
| Connection dialog: MySQL card **Active**, network fields, port `3306`, TLS `Require` | criterion 2 — MySQL is reachable from the UI |
| `draft_to_domain` keeps passwords for non-SQLite drivers | MySQL saves previously lost the password (Postgres-only branch) |
| SSH card stays PostgreSQL-only | no MySQL SSH path in v0.1 |

## Tests

- `bind_params_accepts_common_scalar_types` / `bind_params_rejects_interval_and_inet`
- `mysql_capabilities_advertise_positional_parameters_when_binder_exists`
- `mysql_advertisement_matches_the_shipping_code_paths` (updated)
- `selecting_mysql_sets_port_and_tls_and_preserves_password_on_submit`
- `editing_a_mysql_connection_keeps_the_mysql_driver`
- `mysql_connection_resolves_to_its_own_capability_set` (now expects parameters=true)

`cargo test --workspace` — exit 0, 0 failed (UI lib 417 passed).

## Still open on #235 / #234

- Table-data / data-diff end-to-end qualification for MySQL
- Capability-level *reason* channel (#234 criterion 3)
- Schema/lifecycle cases in `provider_contract_conformance.rs` (#234 criterion 4)
- MySQL SSL mode is stored but the connector URL does not yet map rustls modes

## Live binder proof (follow-up)

```text
DATABASE_URL=mysql://root:dbpro_test@127.0.0.1:33306/dbpro_fixture \
  cargo test -p db-pro-infrastructure --test mysql_integration \
  mysql_positional_parameters_bind_and_round_trip -- --ignored
# ok — dialect `?`/`order`, bound INSERT+SELECT round-trip for int/text/decimal/datetime/bool/json
```
