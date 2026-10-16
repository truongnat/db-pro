# Checklist — SQLite Restore WAL Snapshot

- [x] Stage restore through SQLite snapshot rather than copying only the main file.
- [x] Preserve staged validation and target sidecar handling.
- [x] Add regression coverage for committed WAL data during restore.
- [x] SQLite backup module tests, infrastructure check, formatting, and diff validation pass.
- [ ] Rebase, commit, push, and open a PR linked to #466.
