---
schema_version: 1
id: "MEM-PLAN-sqlite-restore-wal-snapshot-7cf6f460fce6"
title: "Plan archive: Sqlite Restore Wal Snapshot"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sqlite-restore-wal-snapshot]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "7cf6f460fce6f0dcf13c673b6f7ad1a1d013aa9eded247624cf2d4d42a67d94b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sqlite-restore-wal-snapshot/PLAN.md"
  - "docs/plans/active/sqlite-restore-wal-snapshot/FINDINGS.md"
  - "docs/plans/active/sqlite-restore-wal-snapshot/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sqlite-restore-wal-snapshot/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — SQLite Restore WAL Snapshot

## Root Cause

Restore validated the source through SQLite (which sees its WAL) but staged the restore with a byte-copy of only the main `.db` file. A later quick-check accepted that structurally valid but incomplete file, silently omitting committed rows held in `-wal`.

## Fix Strategy

Stage restore with the existing `vacuum_into` helper. SQLite reads a consistent committed snapshot across the source database and its WAL and writes a self-contained staged database. The existing staged validation and atomic target replacement then continue unchanged.

## Related Path

Backup already uses `VACUUM INTO` and has WAL/live-transaction regression tests; only restore required this change.


## Source: `VERIFICATION.md`

# Verification — SQLite Restore WAL Snapshot

- `cargo test -p db-pro-infrastructure --lib sqlite_backup::tests`: 9 passed, 0 failed.
- `cargo check -p db-pro-infrastructure`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- Restore snapshots the source through a read-only SQLite connection; regression test restores from a filesystem read-only artifact.
- Regression keeps a WAL-mode source writer open and verifies restore includes a committed row before checkpoint.

