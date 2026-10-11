# Native UI Regression Repairs — Plan

State: RUNTIME_VERIFY
Branch: `main` (owner-mandated workspace workflow)

## Goal

Repair the native UI regressions reported after the egui 2→3 upgrade: theme values not being
respected by legacy widgets, keyboard focus traversal not following the modal input chain, required
fields lacking visible markers/local errors, and decorative borders appearing on surfaces that
should remain quiet.

## Evidence / baseline

- Baseline SHA: `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`.
- A light-theme runtime capture showed `legacy::sidebar_item` using egui's raw `Button` defaults,
  producing rows inconsistent with DB Pro's light component theme.
- `DbProTheme::apply` applied `border_subtle` to every noninteractive widget. Shared input fields
  also painted a resting outline, and dialog contents added separator strokes.
- Modal `trap_focus` could reclaim focus during Tab navigation. Existing tests did not cover both
  forward and reverse traversal through a modal.
- Connection forms displayed validation only at dialog level. Save Query As rejected an empty name
  on submit without required/error feedback; Agent Settings disabled Save for an empty API key
  without marking that field required.
- Connection port feedback promised 1–65535 but accepted `0`.

## Scope

- Use DB Pro theme-aware buttons for the legacy sidebar surface.
- Keep focus traversal inside the active modal while supporting Tab and Shift+Tab.
- Mark fields required when the UI prevents submission without them; show local errors where
  submission validates the value.
- Make port validation agree with its stated 1–65535 range.
- Remove automatic noninteractive strokes, resting outlines on standard text/password fields, and
  dialog separator rules; preserve focused, hovered, error, and intentionally outlined search
  states.
- Capture normal/error connection surfaces in light/dark themes at available viewport sizes.
- Make the connection modal Cancel action cancel the active connection-test request through the runtime worker; keep the dialog open if cancellation dispatch fails.
- Cancel the provider test future without leaving temporary provider resources registered.

## Non-goals

- Do not change saved-connection lifecycle, credential policy, or mutation behavior; cancellation applies only to a connection test.
- Redesign the product palette or introduce a parallel theme system.
- Mark optional connection host, username, or password fields as required.
- Remove intentional borders from focused/error controls, search inputs, or other explicitly framed
  components.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | Test future is dropped by request cancellation; live provider cancellation is not verified |
| SQLite | yes | Test uses an unregistered actor; live provider cancellation is not verified |
| MySQL | yes | Test uses a local pool rather than a registered connection; live provider cancellation is not verified |
| SQL Server | yes | Test uses a local client rather than a registered connection; live provider cancellation is not verified |

The native dialog sends a cancellation command using the active test's request ID. The runtime worker
selects between the provider test future and a request-scoped cancellation signal. Provider test
paths avoid persistent connector registration so dropping an in-flight future does not leave a
registered test connection behind. `DbProTheme` remains the source of semantic visuals; no palette
or provider layer is added.

## Acceptance criteria

- [x] Sidebar rows use DB Pro theme-aware button variants in light and dark themes.
- [x] Modal traversal test covers Tab and Shift+Tab between fields without leaving the modal.
- [x] Required markers/error feedback match submission requirements; zero ports are rejected by the
  existing 1–65535 rule.
- [x] Resting standard inputs, global noninteractive widgets, and dialog contents no longer gain
  unintended strokes; explicit search/focus/error outlines remain.
- [x] Modal Cancel and window close cancel the active test request; failed dispatch leaves the dialog open.
- [x] Runtime cancellation drops the test future, removes its request registration, and closes the in-flight provider-protocol socket in a worker smoke test.
- [x] SQLite test uses a standalone actor, so cancellation cannot leave a connector-registered test actor.
- [x] Final pinned Rust gates and complete app build pass.
- [ ] Live cancellation is verified independently against PostgreSQL, SQLite, MySQL, and SQL Server.
- [ ] Native screenshots reviewed at exact 1280×800, 1440×900, and 1920×1080 in light/dark normal and error states; current host caps large captures at 838 logical pixels high.
- [ ] Live keyboard interaction and independent review recorded before COMPLETED.
