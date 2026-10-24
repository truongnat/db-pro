---
schema_version: 1
id: "MEM-SRC-b24cc40c0e13515c"
title: "Source mirror: docs/plans/active/safe-sample-sqlite-seed/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/safe-sample-sqlite-seed/PLAN.md"
source_sha256: "b24cc40c0e13515c5159c35cd1c157ca80effe2ac3c037b96845e3691a4946de"
mirror_body_sha256: "b24cc40c0e13515c5159c35cd1c157ca80effe2ac3c037b96845e3691a4946de"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — Safe SQLite Sample Seeding

## Objective

Fix issue #492 so startup never registers an arbitrary or symlinked `/tmp` file as the writable Sample E-Commerce database.

## Target Branch

`fix/safe-sample-sqlite-seed`

## Severity

P1 — startup could silently expose a user-owned SQLite database as a writable demo connection.

## Scope

- Store the fixture under the app's resolved per-user data directory.
- Create the fixture only at a no-overwrite path; validate app id and fixture marker before reuse.
- Reject symlinks/unrecognized existing files and keep the seeded connection read-only.
- Migrate the prior built-in `/tmp/db_pro_sample.db` connection entry to the verified per-user fixture.
- Add regression coverage for existing files, symlinks, idempotence, and read-only configuration.
