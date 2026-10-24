---
schema_version: 1
id: "MEM-SRC-b17adbab459cdc72"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/20-postgres-container-shutdown.txt"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/20-postgres-container-shutdown.txt"
source_sha256: "b17adbab459cdc722cf4da4e5a57251e029bf40d0a911949e21362a4f45da707"
mirror_body_sha256: "b17adbab459cdc722cf4da4e5a57251e029bf40d0a911949e21362a4f45da707"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
### PostgreSQL container — shutdown record (deliverable 3 step 5)

$ docker ps (before stop)
  dbpro-v01-pg-fixture | postgres:16 | Up 8 minutes | 0.0.0.0:55432->5432/tcp, [::]:55432->5432/tcp
  qltx-platform-postgres | postgres:18.2 | Up 26 hours (healthy) | 0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp
  qltx-platform-redis | redis:7.4 | Up 26 hours (healthy) | 0.0.0.0:6379->6379/tcp, [::]:6379->6379/tcp
  qltx-platform-mailpit | axllent/mailpit:v1.27.3 | Up 26 hours (healthy) | 0.0.0.0:1025->1025/tcp, [::]:1025->1025/tcp, 0.0.0.0:8025->8025/tcp, [::]:8025->8025/tcp
  qltx-platform-rabbitmq | rabbitmq:4-management | Up 26 hours (healthy) | 0.0.0.0:5672->5672/tcp, [::]:5672->5672/tcp, 0.0.0.0:15672->15672/tcp, [::]:15672->15672/tcp

$ docker stop dbpro-v01-pg-fixture
dbpro-v01-pg-fixture
  exit=0

$ docker ps -a (after stop — the container still EXISTS in 'Exited' state, kept for re-runs)
  dbpro-v01-pg-fixture | postgres:16 | Exited (0) Less than a second ago | 

$ docker ps (running) — confirm the qltx containers and everything else are untouched
  qltx-platform-postgres | Up 26 hours (healthy) | 0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp
  qltx-platform-redis | Up 26 hours (healthy) | 0.0.0.0:6379->6379/tcp, [::]:6379->6379/tcp
  qltx-platform-mailpit | Up 26 hours (healthy) | 0.0.0.0:1025->1025/tcp, [::]:1025->1025/tcp, 0.0.0.0:8025->8025/tcp, [::]:8025->8025/tcp
  qltx-platform-rabbitmq | Up 26 hours (healthy) | 0.0.0.0:5672->5672/tcp, [::]:5672->5672/tcp, 0.0.0.0:15672->15672/tcp, [::]:15672->15672/tcp

$ named volumes created by this run (anonymous volume for the fixture data dir;
  NOT the compose 'pg_data' volume, which is untouched):
  volume eddfec3b6fe5b5b62179d2c6ab7a262465da7c26f4e7b960e1affdbcd2201a21 -> /var/lib/postgresql/data
  bind  -> /docker-entrypoint-initdb.d/001_schema.sql
  bind  -> /docker-entrypoint-initdb.d/002_seed.sql
  

$ compose volume status (must be untouched / not created by this run):
  main_pg_data volumes found: 0

$ restart instructions for a re-run:
  docker start dbpro-v01-pg-fixture   # the fixture data plus the three restore-target
                                      # databases created in this run
                                      # (dbpro_v01_restore_target,
                                      #  dbpro_v01_restore_pg16client,
                                      #  dbpro_v01_restore_plain)
                                      # persist inside the container's own anonymous volume
  postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture
