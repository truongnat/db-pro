# Native UI Regression Repairs — Findings

## Evidence baseline

- Baseline source SHA: `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`.
- Fixed implementation source SHA: `92746074431b60f2f96347c940293da10952b292`; source pointers below refer to that commit.

## P0

None identified.

## P1

None identified.

## P2

- **Theme mismatch in the legacy sidebar.** At baseline SHA
  `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`, `crates/ui/src/components/legacy.rs::sidebar_item`
  used egui's raw `Button` styling; a light Query runtime capture showed dark row styling
  inconsistent with the light theme. Implementation SHA
  `92746074431b60f2f96347c940293da10952b292` uses the existing Secondary/Ghost variants
  (`legacy.rs:286-307`).
- **Modal keyboard traversal can be interrupted.** At baseline SHA
  `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`, `modal_guard::trap_focus` reclaimed focus when egui
  was processing Tab. Implementation SHA `92746074431b60f2f96347c940293da10952b292` yields on Tab
  presses and reclaims focus outside the modal on the next pass (`modal_guard.rs:72-94`); its test
  covers both directions (`modal_guard.rs:209-347`).
- **Required fields lacked visible and local feedback.** At baseline SHA
  `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`, connection errors were dialog-level only; Save Query
  As rejected an empty name without required/error feedback; Agent Settings' disabled Save button
  was the only API-key requirement signal. Implementation SHA
  `92746074431b60f2f96347c940293da10952b292` adds connection feedback
  (`form_fields.rs:36-53`; `advanced_panels.rs:232-260`), Save Query As validation
  (`query_dialogs_view.rs:220-224`; `query_save_dialog_surface_view.rs:30-38`), and the Agent key
  marker (`agent_settings_view.rs:119-129`).
- **Port-zero validation contradicted its own message.** At baseline SHA
  `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`, the validator promised ports 1–65535 but accepted
  `0`. Implementation SHA `92746074431b60f2f96347c940293da10952b292` shares a `1..=65535` predicate
  for network and SSH ports (`logic.rs:8-20`).
- **Decorative borders were over-applied.** At baseline SHA
  `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`, `DbProTheme::apply` stroked noninteractive widgets,
  standard fields outlined at rest, and dialog content inserted separator lines. Implementation
  SHA `92746074431b60f2f96347c940293da10952b292` removes default strokes
  (`theme.rs:439-447`), leaves standard fields unoutlined at rest
  (`components/input/layout.rs:23-54`), preserves the search-field outline
  (`components/input/search.rs:153-163`), and omits dialog footer rules
  (`components/dialog/frame.rs:27-32`).

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
- Release capture build passes. In a 1280×800 light/dark normal/error run, all four screenshots
  were written, but three processes returned `-6` after rendering. The available crash report points
  to AppKit NSTouchBar/KVO invalidation during the run loop; root cause is not established.
- Live keyboard interaction and independent review remain pending. The selected Orca CLI invocation
  failed with `command not found: orca`; automated Tab/Shift+Tab coverage passes.
