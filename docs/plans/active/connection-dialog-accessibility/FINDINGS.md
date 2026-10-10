# Native UI Regression Repairs — Findings

## Evidence baseline

Source claims below are against baseline SHA `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`; runtime
captures are from the corresponding uncommitted worktree after the changes.

## P0

None identified.

## P1

None identified.

## P2

- **Theme mismatch in the legacy sidebar.** `crates/ui/src/components/legacy.rs::sidebar_item`
  used egui's raw `Button` styling rather than DB Pro's theme-aware `Button`; a light Query
  workspace capture showed dark row styling inconsistent with the light theme. It now uses the
  existing Secondary/Ghost variants.
- **Modal keyboard traversal can be interrupted.** `crates/ui/src/components/dialog/modal_guard.rs::trap_focus`
  reclaimed focus when egui was processing Tab. It now yields during Tab presses and checks
  containment on the following pass; the regression test covers forward Tab and reverse Shift+Tab.
- **Required fields lacked visible and local feedback.** Connection form requirements were only
  reported by a dialog-level error; Save Query As rejected an empty name without marking the field
  or displaying an inline error; Agent Settings' API-key requirement was implicit in its disabled
  Save button. Required markers and local errors are now shown where the value is validated; the
  API-key label is marked required.
- **Port-zero validation contradicted its own message.** The existing connection validator promised
  ports 1–65535 but accepted `0`. Network and SSH validation now share a `1..=65535` predicate.
- **Decorative borders were over-applied.** `DbProTheme::apply` set a stroke for all noninteractive
  widgets, standard input fields outlined at rest, and dialogs inserted separator lines. The global
  stroke is now none; standard fields use fill at rest and reserve outlines for hover/focus/error;
  explicit search outlines and explicitly framed components remain.

## Rejected / downgraded findings

- Connection host, username, and password are not marked required because existing validation does
  not require them.
- Agent API-key submission remains disabled when the field is blank, so it cannot create an invalid
  submission state; the label makes the requirement explicit.

## Runtime gaps

- PostgreSQL: N/A; no provider behavior changed.
- SQLite: N/A; no provider behavior changed.
- Connection light/dark normal/error captures reviewed at 1280×800, 1440×838, and 1920×838; the
  macOS host caps the latter viewport heights at 838 logical pixels.
- Query workspace light/dark captures reviewed at 1280×800. Save Query As and Agent Settings were
  verified through source/build, not separate runtime captures.
- Live keyboard interaction and independent review remain pending. The selected Orca CLI invocation
  failed with `command not found: orca`; automated Tab/Shift+Tab coverage passes.
