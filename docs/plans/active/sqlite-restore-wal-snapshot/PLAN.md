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
