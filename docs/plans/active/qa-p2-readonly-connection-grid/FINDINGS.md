# FINDINGS — QA-P2-08 Read-Only Connection Data Grid Visual Affordance

## Severity: P2

## Finding
`DataSection` and `DataGrid` in the frontend checked `pkColumns.length > 0` to decide if cell editing / row deletion is allowed, but ignored `connection.readonly`.

## Impact
Users connected to a read-only database connection could double-click cells, stage cell edits, stage row deletes, and open row edit dialogs. Applying these staged changes called the backend API which failed with "Connection is read-only".

## Remediation
1. `DataSection` reads `readonly` from `useConnectionStore`.
2. Guarded staged mutation callbacks in `DataSection` if `isReadonlyConnection` is true.
3. Updated `DataGrid` to calculate `canEdit = pkColumns.length > 0 && !isReadonlyConnection`.
4. Rendered connection read-only banner (`dataGrid.readOnlyConnection`).
5. Added unit tests and verified all quality gates.
