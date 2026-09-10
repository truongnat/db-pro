# RC1 P2 Grid & Connection Correctness Remediation

## Executive Summary
This feature branch addresses three confirmed P2 defects in DB Pro's Data Grid and Connection management UI:
- **QA-P2-07**: Columns picker double-toggle in Data Grid toolbar.
- **QA-P2-15**: Stale Connection Test result badge on form input modification.
- **QA-P2-16**: Test Connection error details hidden from user.

## Lifecycle
`PLANNING → IMPLEMENTING → REVIEW → COMPLETED`

## Scope & Implementation Details

### 1. Data Grid Columns Picker Double-Toggle (QA-P2-07)
- **Problem**: In `DataToolbar`, clicking directly on the `<Checkbox>` for a column triggers `onCheckedChange` and then bubbles a `click` event to the wrapper `div`, calling `onToggleHiddenColumn` twice and effectively cancelling the action.
- **Fix**: In `frontend/src/modules/data-grid/components/data-toolbar.tsx`, stop propagation on the checkbox click event or handle column visibility toggle cleanly without duplicate callback invocations.

### 2. Connection Test Stale State & Error Display (QA-P2-15 & QA-P2-16)
- **Problem**:
  - `ConnectionEditor` retains a previous `testResult` ("success" or "error") even after the user modifies host, port, database, username, password, or SSH settings.
  - `ConnectionDialog` suppresses the structured backend `userMessage` on test failures and renders only generic `connection.testFailed`.
- **Fix**:
  - Reset `testResult` and `testErrorDetail` in `ConnectionEditor` state whenever form inputs (`formData`, `password`, `showSsh`) are mutated.
  - Add `testErrorDetail` prop to `ConnectionEditor` and display the specific backend error message when `testResult === "error"`.
  - Pass `(err as { userMessage?: string }).userMessage` from `useTestConnection` mutation in `ConnectionDialog` to `ConnectionEditor`.
