# Checklist

- [x] Create focused branch and preserve pre-existing worktree changes.
- [x] Add stable PK identity matching to staged mutations.
- [x] Allow PK edits while retaining original PK predicate values.
- [x] Merge one row's edited cells into one update mutation.
- [x] Disable Apply while a validation error or apply request is active.
- [x] Add conflict and invariant-violation outcomes for affected-row counts.
- [x] Preserve staged rows after rollback/conflict and expose recovery actions.
- [x] Add regression tests for single/composite PK, PK edits, ChangeSet
      transitions, and mutation failure cleanup.
- [ ] Verify PostgreSQL and SQLite independently at runtime.
- [x] Run Rust quality gates and native release build.
- [ ] Collect native UI evidence at required viewport/state matrix.
- [x] Record remaining P0/P1/P2 and known limitations.
