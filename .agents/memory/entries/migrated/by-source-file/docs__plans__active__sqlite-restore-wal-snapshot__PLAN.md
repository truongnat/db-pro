---
schema_version: 1
id: "MEM-SRC-e080854dfbc15a4d"
title: "Source mirror: docs/plans/active/sqlite-restore-wal-snapshot/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-restore-wal-snapshot/PLAN.md"
source_sha256: "e080854dfbc15a4d9f45b1df7490f04e8c3227f3f90fa5c911ef57118144eaf0"
mirror_body_sha256: "e080854dfbc15a4d9f45b1df7490f04e8c3227f3f90fa5c911ef57118144eaf0"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — SQLite Restore WAL Snapshot

## Objective

Fix issue #466 so a SQLite restore includes committed data still residing in the source `-wal` file.

## Target Branch

`fix/sqlite-wal-backup-restore`

## Severity

P1 — restore could report success while silently omitting committed rows.

## Scope

- Replace main-file copy during restore with a consistent SQLite snapshot via the existing `VACUUM INTO` helper.
- Preserve current staged-file validation and target sidecar cleanup/publication semantics.
- Add a WAL-mode regression with a live writer and an uncheckpointed committed row.
