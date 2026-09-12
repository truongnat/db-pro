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

The core focused suite currently passes 20 tests, including zero-row
`update_row`/`delete_row` rejection and transaction failure metadata. UI tests
cannot compile until the current dirty changes are completed.
