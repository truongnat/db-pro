# Findings

## Initial evidence

1. `ChangeSet` currently keys staged updates/deletes by `row_index`. This is a
   UI coordinate and can change after filtering, sorting, paging, or reload.
2. `row_identity` reads PK values from the server result, but the ChangeSet
   does not own a stable identity independent of that coordinate.
3. `TableDataService::update_row` rejects updates that include PK columns, so
   the required `WHERE original PK` / `SET final PK` behavior is unavailable.
4. The transaction statement contract already has `max_affected_rows`, but
   zero-row conflict classification and the `>1` invariant need explicit
   domain/UI mapping.
5. The current dirty worktree does not compile: UI pattern matches omit the new
   `MutationTarget` fields and `pk_columns`/`pk_values` are moved while building
   failure targets.

## Severity

- P1: a staged mutation keyed only by row coordinate can target the wrong row
  after a grid state change.
- P1: PK edits are rejected instead of producing the required safe mutation.
- P1: zero affected rows are not yet represented as a first-class conflict in
  the mutation result contract.

## Existing coverage

The core focused suite currently passes 21 tests, including zero-row
`update_row`/`delete_row` rejection and transaction failure metadata. The UI
suite now compiles and passes the identity, typed-input, sorting, and conflict
recovery regressions listed in the plan.

## Completion-slice review

- Row identity is now a domain value owned by the ChangeSet; `current_row_index`
  is optional targeting metadata for focus/render only.
- The visible grid builds row/column coordinate maps once per visible layout;
  no per-cell `.position()` lookup remains in the render/navigation path.
- Conflict reload keeps staged mutations and Retry waits for the refreshed
  result before applying again. Constraint failures retain the staged values.
- Remaining P1: PostgreSQL and SQLite provider flows still need independent
  runtime evidence with a live concurrent change.
- Remaining P2: Reload Row fetches the original PK predicate and merges a
  returned row into the current result when visible; a deliberate Keep Local /
  Use Server Values choice is still not exposed because staged values remain
  the safe default.
- Remaining P2: provider-specific structure metadata fields (index method,
  INCLUDE/predicate/definition and full FK actions) are not expanded in this
  slice; binary editing, count caching, and state extraction from DbProApp also
  remain pending.
