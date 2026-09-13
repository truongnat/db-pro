# Checklist

- [x] Create focused branch and preserve pre-existing worktree changes.
- [x] Add stable PK identity matching to staged mutations.
- [x] Allow PK edits while retaining original PK predicate values.
- [x] Merge one row's edited cells into one update mutation.
- [x] Disable Apply while a validation error or apply request is active.
- [x] Add conflict and invariant-violation outcomes for affected-row counts.
- [x] Preserve staged rows after rollback/conflict and expose recovery actions.
- [x] Keep conflict reloads staged and support retry after the refreshed result.
- [x] Normalize mutation error categories and render conflict rows distinctly.
- [x] Store table-data sorting as ordered clauses with Shift+click support.
- [x] Persist grid layout state by connection/schema/table and add layout actions.
- [x] Replace per-cell visible-coordinate scans with prebuilt lookup maps.
- [x] Gate filter operators by column datatype and validate before DB dispatch.
- [x] Support multiple AND filters, editable/removable chips, and pagination reset.
- [x] Support server-side multi-sort cycle, priority display, and sort guards.
- [x] Normalize persisted layout after column add/remove/order changes.
- [x] Add header Add Filter, separator auto-size, expanded cell editor, and value tooltips.
- [x] Add pending-change review with old/new diff and per-entry revert actions.
- [x] Support known/unknown-total pagination and page-size selection.
- [x] Add regression tests for single/composite PK, PK edits, ChangeSet
      transitions, and mutation failure cleanup.
- [ ] Verify PostgreSQL and SQLite independently at runtime.
- [x] Run Rust quality gates and native release build.
- [ ] Collect native UI evidence at required viewport/state matrix.
- [ ] Expand structure metadata fields and provider-specific metadata tests.
- [x] Reload conflict rows by RowIdentity without replacing the whole table result.
- [ ] Extract SelectionState/TableQueryState/GridLayoutState/MutationState from DbProApp.
- [x] Record remaining P0/P1/P2 and known limitations.
