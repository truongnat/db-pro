---
schema_version: 1
id: "MEM-SRC-fe81b065cb23a024"
title: "Source mirror: docs/plans/active/connection-dialog-accessibility/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/connection-dialog-accessibility/FINDINGS.md"
source_sha256: "fe81b065cb23a024a03b001fc5e39e3aecb57209d0079a1c380c91735f8797ef"
mirror_body_sha256: "fe81b065cb23a024a03b001fc5e39e3aecb57209d0079a1c380c91735f8797ef"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Native UI Regression Repairs — Findings

## Evidence baseline

- Baseline source SHA for the egui regression repairs: `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`.
- Original UI implementation SHA: `92746074431b60f2f96347c940293da10952b292`.
- Connection-test cancellation implementation SHA: `551a35f7c159abc27cf97ac4d0f8bef86a002891`.

## P0

None identified.

## P1

None identified.

## P2
- **Connection-test Cancel only closed the modal.** At source SHA
  `1888e812360c2acb59cdb9c7af88582ba0290725`, the footer's Cancel action closed the dialog and
  cleared UI pending state, while `crates/runtime/src/worker.rs` ran `connection_api.test` in a
  detached task with no cancellation command. Closing during a slow provider test left the request
  running. At implementation SHA `551a35f7c159abc27cf97ac4d0f8bef86a002891`, Cancel and window
  close dispatch `CancelTestConnection` with the active request ID
  (`crates/ui/src/connection/view.rs:236-249`); the worker selects the test future against a
  request-scoped oneshot and suppresses completion after cancellation
  (`crates/runtime/src/worker.rs:1471-1519,2715-2720`). Failed dispatch leaves the dialog open.
  PostgreSQL uses the existing local pool path, SQLite uses a standalone actor, MySQL a local pool,
  and SQL Server a local client, avoiding persistent connector registration for temporary tests.
- **No request-specific cancellation control while testing.** At source SHA
  `551a35f7c159abc27cf97ac4d0f8bef86a002891`, `draw_connection_footer` kept the Test Connection
  control in its loading state while the separate modal Cancel action closed the form
  (`crates/ui/src/connection/view.rs:383-403, 435-442`). The owner reported no visible Cancel
  action for the test request. Fixed at source SHA `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`:
  pending tests show a localized Cancel Test button; its handler dispatches the active request ID,
  clears pending test UI state, and keeps the dialog open
  (`crates/ui/src/connection/view.rs:236-263, 397-435`; `crates/ui/src/connection/tests.rs:261-292`).
  Light/dark captures at all requested widths show the separate action.

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

- PostgreSQL: worker cancellation closes a stalled local PostgreSQL-protocol socket; no live database test.
- SQLite: standalone actor; live provider cancellation not tested.
- MySQL: local pool; live provider cancellation not tested.
- SQL Server: local client; live provider cancellation not tested.
- Connection light/dark normal/error captures reviewed at 1280×800, 1440×838, and 1920×838; the
  macOS host caps the latter viewport heights at 838 logical pixels.
- Query workspace light/dark captures reviewed at 1280×800. Save Query As and Agent Settings were
  verified through source/build, not separate runtime captures.
- Release capture build passes. In a 1280×800 light/dark normal/error run, all four screenshots
  were written, but three processes returned `-6` after rendering. The available crash report points
  to AppKit NSTouchBar/KVO invalidation during the run loop; root cause is not established.
- Live keyboard interaction and independent review remain pending. The selected Orca CLI invocation
  failed with `command not found: orca`; automated Tab/Shift+Tab coverage passes.
