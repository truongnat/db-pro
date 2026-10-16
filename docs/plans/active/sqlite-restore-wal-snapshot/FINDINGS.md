# Findings — SQLite Restore WAL Snapshot

## Root Cause

Restore validated the source through SQLite (which sees its WAL) but staged the restore with a byte-copy of only the main `.db` file. A later quick-check accepted that structurally valid but incomplete file, silently omitting committed rows held in `-wal`.

## Fix Strategy

Stage restore with the existing `vacuum_into` helper. SQLite reads a consistent committed snapshot across the source database and its WAL and writes a self-contained staged database. The existing staged validation and atomic target replacement then continue unchanged.

## Related Path

Backup already uses `VACUUM INTO` and has WAL/live-transaction regression tests; only restore required this change.
