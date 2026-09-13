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
- [x] Persist layout by stable column name, migrate safe legacy layouts, and
      drop stale schema entries without applying renamed-column state.
- [x] Add header Add Filter, separator auto-size, expanded cell editor, and value tooltips.
- [x] Add grouped pending-change review by RowIdentity with old/new diff,
      Revert Cell/Row, Undo Delete, and temporary insert identity actions.
- [x] Support known/unknown-total pagination and page-size selection.
- [x] Add regression tests for single/composite PK, PK edits, ChangeSet
      transitions, and mutation failure cleanup.
- [ ] Verify PostgreSQL and SQLite independently at runtime.
- [x] Run Rust quality gates and native release build.
- [ ] Collect native UI evidence at required viewport/state matrix.
- [x] Expand structure metadata fields and catalog-backed PostgreSQL/SQLite
      mappings for columns, indexes, and foreign-key actions.
- [ ] Add live PostgreSQL metadata/runtime verification for the expanded fields.
- [x] Add requested 1k/10k-row, 50-column visual-map benchmark cases.
- [x] Reload conflict rows by RowIdentity without replacing the whole table result.
- [ ] Extract SelectionState/TableQueryState/GridLayoutState/MutationState from DbProApp.
- [x] Record remaining P0/P1/P2 and known limitations.
