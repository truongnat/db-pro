---
schema_version: 1
id: "MEM-SRC-cfd5811757248e52"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/13-pg-restore-skew-controls.txt"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/13-pg-restore-skew-controls.txt"
source_sha256: "cfd5811757248e52ece7cb1451b9c9ff5229be3592a2c9d36b698648c3fa44e9"
mirror_body_sha256: "cfd5811757248e52ece7cb1451b9c9ff5229be3592a2c9d36b698648c3fa44e9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
### PostgreSQL backup/restore — proper version-matched control + plain-format path

  Finding so far: an archive written by pg_dump 18.4 (Dump Version 1.16) cannot be read
  by pg_restore 16.15 at all ('unsupported version (1.16) in file header').
  PostgreSQL's own compatibility rule: pg_restore must be same-or-newer than pg_dump.

$ CONTROL A — dump AND restore both with the version-matched 16.15 tools inside the container
pg_dump (PostgreSQL) 16.15 (Debian 16.15-1.pgdg13+2)
dropdb: error: connection to server on socket "/var/run/postgresql/.s.PGSQL.5432" failed: FATAL:  role "root" does not exist

  restored row counts (version-matched control):
ERROR:  relation "orders" does not exist
LINE 1: SELECT 'orders=' || (SELECT COUNT(*) FROM orders) || ' order...
                                                  ^
  fingerprint (version-matched control):
ERROR:  relation "orders" does not exist
LINE 1: ...r_id, status::text, total, notes, created_at FROM orders) t;
                                                             ^
  003_verify.sql against the version-matched restore:
psql:fixtures/postgres/003_verify.sql:22: ERROR:  relation "categories" does not exist
LINE 1: (SELECT COUNT(*) FROM categories) <> 4
                              ^
QUERY:  (SELECT COUNT(*) FROM categories) <> 4
CONTEXT:  PL/pgSQL function inline_code_block line 3 at IF

$ CONTROL B — the PLAIN-format restore path the app uses (psql -f, see pg_dump.rs:143-147)
NOTICE:  database "dbpro_v01_restore_plain" does not exist, skipping
  psql -f (18.4 client) exit=0
  stderr/stdout lines mentioning transaction_timeout or ERROR:
4:psql:/tmp/dbpro-v01-runtime/dbpro_fixture.sql:13: ERROR:  unrecognized configuration parameter "transaction_timeout"
  plain-format restored row counts:
    orders=4 order_items=6
