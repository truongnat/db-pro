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

## Non-goals

- Change provider behavior, credential policy, or database lifecycle behavior.
- Redesign the product palette or introduce a parallel theme system.
- Mark optional connection host, username, or password fields as required.
- Remove intentional borders from focused/error controls, search inputs, or other explicitly framed
  components.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | N/A | UI-only changes; no provider operation changed |
| SQLite | N/A | UI-only changes; no provider operation changed |

## Architecture

The work stays in native egui presentation and existing form validation/state. `DbProTheme` remains
the source of semantic visuals; no palette or provider layer is added. Capture mode selects
deterministic UI state only.

## Acceptance criteria

- [x] Sidebar rows use DB Pro theme-aware button variants in light and dark themes.
- [x] Modal traversal test covers Tab and Shift+Tab between fields without leaving the modal.
- [x] Required markers/error feedback match submission requirements; zero ports are rejected by the
  existing 1–65535 rule.
- [x] Resting standard inputs, global noninteractive widgets, and dialog contents no longer gain
  unintended strokes; explicit search/focus/error outlines remain.
- [ ] Final pinned Rust gates and complete app build pass.
- [ ] Native screenshots reviewed at exact 1280×800, 1440×900, and 1920×1080 in light/dark normal
  and error states. Current host caps large captures at 838 logical pixels high.
- [ ] Live keyboard interaction and independent review recorded before COMPLETED.
