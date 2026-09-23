# RC1 P2 Grid & Connection Correctness Findings

## QA-P2-07 — Data Grid Columns picker double-toggles on direct checkbox click
- **Severity**: P2
- **Area**: Data Grid / Data Toolbar
- **Files**: `frontend/src/modules/data-grid/components/data-toolbar.tsx`
- **Source Evidence**: Wrapper `div` has `onClick={() => onToggleHiddenColumn(c.name)}` and child `<Checkbox>` has `onCheckedChange={() => onToggleHiddenColumn(c.name)}`. Clicking `<Checkbox>` invokes `onCheckedChange` and then bubbles `onClick` to the `div`, calling `onToggleHiddenColumn` twice.
- **Failure Scenario**: User opens Columns menu in Data Grid toolbar, clicks directly on the checkbox for a column. Column toggles hide and immediately unhide, remaining unchanged.

## QA-P2-15 — Connection Test success/error state becomes stale after form edits
- **Severity**: P2
- **Area**: Connection Editor
- **Files**: `frontend/src/modules/connection/components/connection-editor.tsx`
- **Source Evidence**: `testResult` is passed from parent state or stored in prop, but editing form fields (`formData`, `password`, `sshTunnel`) does not clear `testResult`.
- **Failure Scenario**: User tests connection successfully ("Test Connection Success"). User then edits Host to an invalid address. The green success badge remains visible despite the edited form configuration no longer being tested.

## QA-P2-16 — Test Connection hides useful backend error detail
- **Severity**: P2
- **Area**: Connection Dialog / Connection Editor
- **Files**:
  - `frontend/src/modules/connection/components/connection-dialog.tsx`
  - `frontend/src/modules/connection/components/connection-editor.tsx`
- **Source Evidence**: `handleTest` in `ConnectionDialog` calls `snackbar.error(t("connection.testFailed"))` on error and passes `testResult="error"` without passing error details (`err.userMessage`).
- **Failure Scenario**: Test connection fails due to invalid password or port mismatch. User receives generic "Test Connection Failed" with no indication of why (e.g. "password authentication failed for user postgres").
