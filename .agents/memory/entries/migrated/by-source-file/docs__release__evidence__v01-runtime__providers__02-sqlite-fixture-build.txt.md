---
schema_version: 1
id: "MEM-SRC-e2f9105fba3ba349"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/02-sqlite-fixture-build.txt"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/02-sqlite-fixture-build.txt"
source_sha256: "e2f9105fba3ba3498ce5b0a63154bb16990e2b1484301ba9c55815d6c723b832"
mirror_body_sha256: "e2f9105fba3ba3498ce5b0a63154bb16990e2b1484301ba9c55815d6c723b832"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
### SQLite fixture build — raw output
$ sqlite3 --version
3.51.0 2025-06-12 13:14:41 f0ca7bba1c5e232e5d279fad6338121ab55af0c8c68c84cdfb18ba5114dcaapl (64-bit)

$ sqlite3 /tmp/dbpro-v01-runtime/sqlite-runtime.db < fixtures/smoke/sqlite/runtime_fixture.sql
exit=0

$ ls -l /tmp/dbpro-v01-runtime/sqlite-runtime.db
-rw-r--r--@ 1 truongdev  wheel  81920 Sep 14 22:58 /tmp/dbpro-v01-runtime/sqlite-runtime.db
