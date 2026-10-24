---
schema_version: 1
id: "MEM-SRC-7d60f4dd129ebbc7"
title: "Source mirror: docs/plans/active/sqlite-restore-wal-snapshot/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-restore-wal-snapshot/FINDINGS.md"
source_sha256: "7d60f4dd129ebbc76b7ddde51eeb9cd0126a78173f7c9d691de4f8afe978d924"
mirror_body_sha256: "7d60f4dd129ebbc76b7ddde51eeb9cd0126a78173f7c9d691de4f8afe978d924"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — SQLite Restore WAL Snapshot

## Root Cause

Restore validated the source through SQLite (which sees its WAL) but staged the restore with a byte-copy of only the main `.db` file. A later quick-check accepted that structurally valid but incomplete file, silently omitting committed rows held in `-wal`.

## Fix Strategy

Stage restore with the existing `vacuum_into` helper. SQLite reads a consistent committed snapshot across the source database and its WAL and writes a self-contained staged database. The existing staged validation and atomic target replacement then continue unchanged.

## Related Path

Backup already uses `VACUUM INTO` and has WAL/live-transaction regression tests; only restore required this change.
