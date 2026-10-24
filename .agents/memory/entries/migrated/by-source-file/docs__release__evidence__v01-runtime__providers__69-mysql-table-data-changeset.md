---
schema_version: 1
id: "MEM-SRC-9b86e659c245c96a"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/69-mysql-table-data-changeset.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/69-mysql-table-data-changeset.md"
source_sha256: "9b86e659c245c96a27840c72acd256db33b4f5baa2f665861e2b8b4c1ef6b514"
mirror_body_sha256: "9b86e659c245c96a27840c72acd256db33b4f5baa2f665861e2b8b4c1ef6b514"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# MySQL parameterized change-set path (#235 criterion 5)

## Claim

MySQL table mutations use the canonical `sql_builder` + `execute_parameterized_transaction`
path (same as PostgreSQL/SQLite table-data apply), not ad-hoc SQL.

## Landed

- `MySqlConnector::execute_parameterized_transaction` — begin/bind/execute/rollback on zero
  affected rows / max-row invariant / commit
- Live `mysql_table_data_change_set_round_trips`: insert → select → update → delete via
  `CompositeConnector` + MySQL dialect + `sql_builder`

## Command

```bash
DATABASE_URL=mysql://root:…@127.0.0.1:33306/dbpro_fixture \
  cargo test -p db-pro-infrastructure --test mysql_integration \
  mysql_table_data_change_set_round_trips -- --ignored
```

## Result

`ok` against `mysql:8.4` container `db-pro-mysql-1`.
