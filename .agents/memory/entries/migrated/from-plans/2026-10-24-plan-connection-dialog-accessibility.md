---
schema_version: 1
id: "MEM-PLAN-connection-dialog-accessibility-6b0bc0f5c8cd"
title: "Plan archive: Connection Dialog Accessibility"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, connection-dialog-accessibility]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "6b0bc0f5c8cd685066d8f7a931835a67fa7ecd9462e1cb2db74647eb64cb1401"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/connection-dialog-accessibility/PLAN.md"
  - "docs/plans/active/connection-dialog-accessibility/FINDINGS.md"
  - "docs/plans/active/connection-dialog-accessibility/VERIFICATION.md"
  - "docs/plans/active/connection-dialog-accessibility/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/connection-dialog-accessibility/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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
- While a test request is pending, show a dedicated localized Cancel Test action that cancels the
  request while keeping the connection modal open; preserve the separate modal Cancel action.

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
- [x] Pending-test footer replaces its spinner-only test control with Cancel Test; cancellation
  dispatch is covered by a UI regression test and the modal remains open, while modal Cancel closes.
- [x] Runtime cancellation drops the test future, removes its request registration, and closes the in-flight provider-protocol socket in a worker smoke test.
- [x] SQLite test uses a standalone actor, so cancellation cannot leave a connector-registered test actor.
- [x] Final pinned Rust gates and complete app build pass.
- [ ] Live cancellation is verified independently against PostgreSQL, SQLite, MySQL, and SQL Server.
- [ ] Native screenshots reviewed at exact 1280×800, 1440×900, and 1920×1080 in light/dark normal and error states; current host caps large captures at 838 logical pixels high.
- [ ] Live keyboard interaction and independent review recorded before COMPLETED.


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Native UI Regression Repairs — Verification

State: RUNTIME_VERIFY  
Baseline source SHA: `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`.  
Implementation source SHA: `92746074431b60f2f96347c940293da10952b292` (`fix(ui): repair post-egui-upgrade regressions`).

## Commands actually executed

```text
cargo +1.95.0 fmt --all -- --check
PASS, exit 0

cargo +1.95.0 check --workspace
PASS, exit 0

cargo +1.95.0 clippy --workspace --all-targets -- -D warnings
PASS, exit 0

cargo +1.95.0 test --locked -p db-pro-ui --lib
PASS: 1,124 passed / 0 failed / 0 ignored, exit 0

cargo +1.95.0 test --workspace
PASS: 1,788 passed / 0 failed / 41 ignored (22 suites), exit 0

cargo +1.95.0 build --release --locked -p db-pro-native
PASS, exit 0

cargo +1.95.0 build --locked -p db-pro-native --features capture
PASS, exit 0

cargo +1.95.0 test --locked -p db-pro-ui --lib components::dialog::modal_guard::tests::tab_advances_focus_between_fields_inside_modal -- --nocapture
PASS: 1 passed / 0 failed / 0 ignored, exit 0

bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
PASS: 12 pass / 4 warning categories / 0 errors, exit 0
```

The clean-code warnings are heuristic/ratcheted findings (numeric casts, functions with more than
three parameters, long functions, and long files); the scan reported no blocking errors. The
focused modal test exercises forward Tab and reverse Shift+Tab; egui applies the reverse traversal
on the following frame.

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | N/A | N/A | UI-only changes; provider behavior unchanged |
| SQLite | N/A | N/A | UI-only changes; provider behavior unchanged |

## UI runtime evidence

All captures used `DB_PRO_CAPTURE_SETTLE_FRAMES=90`. The native capture binary exited 0 for each
run.

- Connection dialog, normal/error, light/dark, requested 1280×800:
  `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1280x800.png`
- Connection dialog, normal/error, light/dark, requested 1440×900:
  `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1440x900.png`
- Connection dialog, normal/error, light/dark, requested 1920×1080:
  `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1920x1080.png`
- Query workspace light/dark:
  `/tmp/ui-regression-query-{light,dark}-1280x800.png`

The 1280×800 captures rendered at 2560×1600 physical pixels. The 1440×900 and 1920×1080
requests rendered at 2880×1676 and 3840×1676 physical pixels, respectively: this host caps the
logical viewport height at 838, so exact requested heights remain unavailable. Normal/error
connection captures and both Query themes were visually inspected. Inline name, database, and
port-zero errors and required markers are visible; the light Query sidebar follows the shared
theme. Error captures at the capped larger sizes clip some lower helper content.

### Follow-up release capture run — 2026-10-11

`cargo +1.95.0 build --release --locked -p db-pro-native --features capture` passed. The capture
feature emitted two dead-code warnings: `AUDIT_TIMEOUT_FRAMES` and `CaptureApp::{audit, audit_done}`.
Four release-binary captures were launched at 1280×800 and visually rendered:

- Light normal: exit 0.
- Light error, dark normal, dark error: process return `-6` (SIGABRT) after writing screenshots.
  Images: `/tmp/db-pro-release-run-{normal,error}-{light,dark}-1280x800.png`.
- The available crash report
  `/Users/truongdq/Library/Logs/DiagnosticReports/db-pro-native-2026-10-11-073843.ips` records a
  main-thread exception in AppKit's `NSTouchBarFinderObservation` / KVO invalidation path during
  the NSApplication run loop. At implementation SHA
  `92746074431b60f2f96347c940293da10952b292`, `capture.rs:521-525` writes the screenshot then
  requests `ViewportCommand::Close`. The report points to AppKit during that close/run-loop path;
  it is not evidence that the normal release binary exits cleanly. The release capture smoke is
  therefore **partial**, not a four-state pass.

## UI lifecycle

```text
egui theme application → shared theme-aware component styling
modal Tab / Shift+Tab → in-modal input focus chain → containment on next pass
connection/query form submit → existing validation → local field feedback
```

Connection normal/error states were exercised in the native capture runtime. Save Query As and
Agent Settings were verified through source, compilation, and shared input component behavior, but
did not receive separate surface captures. The cancellation follow-up changed provider test
resource lifetimes; live PostgreSQL, SQLite, MySQL, and SQL Server cancellation was not exercised.

## Review

Self-review complete against implementation SHA `92746074431b60f2f96347c940293da10952b292`: no
P0/P1 findings; the addressed presentation/accessibility issues are P2. Independent review and owner keyboard interaction remain pending. The
computer-use skill selected `orca`, but `orca skills get computer-use --json` returned
`command not found: orca`; live keyboard interaction was not available through that route.

## Remaining evidence gaps

- Capture at exact 1440×900 and 1920×1080; the current display caps height at 838 logical pixels.
- Verify real Tab/Shift+Tab interaction in a launched native window.
- Capture Save Query As and Agent Settings surfaces independently.
- Obtain independent review and owner verification.

## Completion decision

Remain RUNTIME_VERIFY. Pinned workspace gates pass and available captures were reviewed, but exact-height captures, live provider cancellation for PostgreSQL/SQLite/MySQL/SQL Server, real keyboard interaction, separate Save Query As / Agent Settings captures, and independent review remain open. Do not mark COMPLETED until all applicable lifecycle gates are satisfied.

## Connection-test cancellation follow-up — 2026-10-11

Implementation source SHA: `551a35f7c159abc27cf97ac4d0f8bef86a002891`.

### Commands and results

- `cargo +1.95.0 fmt --all -- --check` — pass.
- `cargo +1.95.0 check --workspace` — pass.
- `cargo +1.95.0 clippy --workspace --all-targets -- -D warnings` — pass.
- `cargo +1.95.0 test --workspace` — 1,793 passed / 0 failed / 41 ignored across 22 suites.
- `cargo +1.95.0 build --release --locked -p db-pro-native` — pass.
- `cargo +1.95.0 build --release --locked -p db-pro-native --features capture` — pass; two capture-feature dead-code warnings.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` — 12 checks passed, 4 warning categories, 0 errors. Warnings are numeric-cast, function/file size, and clone heuristics.
- `cargo +1.95.0 test --locked -p db-pro-ui --lib cancelling_connection_test_dispatches_cancel_for_the_active_request` — 1 passed.
- `cargo +1.95.0 test --locked -p db-pro-ui --lib failed_connection_test_cancel_dispatch_keeps_the_dialog_open` — 1 passed.
- `cargo +1.95.0 test --locked -p db-pro-runtime --lib cancelling_connection_test_drops_the_active_future` — 1 passed.
- `cargo +1.95.0 test --locked -p db-pro-runtime --lib cancel_test_connection_command_closes_the_pending_provider_socket` — 1 passed; a stalled local PostgreSQL-protocol peer observed EOF and no completion/failure event.
- `cargo +1.95.0 test --locked -p db-pro-native --bin db-pro-native connection_test_cancel_translates_with_the_original_request_id` — 1 passed.
- `cargo +1.95.0 test --locked -p db-pro-infrastructure --test integration test_connection_succeeds` — 1 passed.

### Native capture review

The release capture binary was built with `--features capture`. Light/dark normal/error screenshots
were written for requested 1280×800, 1440×900, and 1920×1080. The host produced effective
1440×838 and 1920×838 logical viewports. Representative light/dark and normal/error captures were
visually reviewed; screenshots show the Cancel action in the unchanged modal footer and field errors
in both themes. Paths: `/tmp/db-pro-cancel-{normal,error}-{light,dark}-{1280x800,1440x900,1920x1080}.png`.

All 12 images were written. Six capture processes exited 0; six returned SIGABRT after writing.
An observed exception is AppKit's `NSTouchBarFinderObservation` removing an unregistered
`nextResponder` observer during the NSApplication run loop. This capture shutdown anomaly is not
attributed to the cancellation change. The capture harness has no connection-test-in-progress
fixture, so loading-state visuals were not captured.

### Remaining gaps

- Live cancellation against PostgreSQL, SQLite, MySQL, and SQL Server remains unverified for each provider.
- Real keyboard interaction, independent review, exact 1440×900 / 1920×1080 viewports, and separate
  Save Query As / Agent Settings captures remain pending.
- No live provider/runtime evidence is inferred from the local PostgreSQL-protocol socket smoke.

### Decision

Remain RUNTIME_VERIFY. The Cancel request now reaches the runtime worker and demonstrably closes an
in-flight provider-protocol socket; live provider behavior and the remaining UI/review gates are open.

## Visible pending-test cancellation action — 2026-10-11

Implementation source SHA: `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`.

### Commands and results

- `cargo +1.95.0 fmt --all -- --check` — pass, exit 0.
- `cargo +1.95.0 test --locked -p db-pro-ui --lib cancel_test_request_keeps_connection_dialog_open -- --nocapture` — 1 passed, 0 failed.
- `cargo +1.95.0 test --locked -p db-pro-ui --lib cancel -- --nocapture` — 14 passed, 0 failed, 1,113 filtered.
- `cargo +1.95.0 build --release --locked -p db-pro-native --features capture` — pass; existing capture-feature dead-code warnings for `AUDIT_TIMEOUT_FRAMES` and `CaptureApp::{audit, audit_done}`.

### Native capture review

The capture fixture sets the UI's pending-test state without contacting a provider. The UI regression
test verifies cancellation dispatches the active request ID and leaves the modal open. Normal and
pending-state screenshots were written and inspected for both themes at requested 1280×800,
1440×900, and 1920×1080:

- Pending: `/tmp/db-pro-test-cancel-pending-{light,dark}-{1280x800,1440x900,1920x1080}.png`.
- Normal: `/tmp/db-pro-test-cancel-normal-{light,dark}-{1280x800,1440x900,1920x1080}.png`.

The dedicated Cancel Test action is visible in all six pending-state captures. Normal captures keep
Test Connection and the separate modal Cancel. The host caps the 1440×900 and 1920×1080 requests at
838 logical pixels high (physical images 2880×1676 and 3840×1676).

All 12 images were written; seven capture processes returned SIGABRT after the screenshot was
written and five exited 0. The abort output records AppKit's `NSTouchBarFinderObservation` removing
an unregistered `nextResponder` observer during shutdown, matching the existing capture limitation.
This is a capture shutdown anomaly, not evidence about the normal app process.

### Remaining gaps

- No live provider test was run or cancelled; provider-specific behavior remains unverified.
- The visible control was reviewed from native capture; live pointer interaction and independent
  review remain pending. `orca` is unavailable in this environment.
- Exact 1440×900 / 1920×1080 logical heights remain unavailable on this host.

### Decision

The request-specific Cancel Test affordance and its UI cancellation path are implemented at
`c02ec5ac9ac42d2c4375f94269f6d773bc642a53`. Keep the feature in RUNTIME_VERIFY for remaining live
provider, live interaction, viewport, and independent-review evidence.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — native UI regressions after egui 2→3

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | OpenAI Codex · native UI regression repair lane |
| Issue(s) | n/a — owner request; no issue number supplied |
| Task state | Done (plan lifecycle: RUNTIME_VERIFY) |
| Baseline SHA | `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`; cancellation-specific baseline: `1888e812360c2acb59cdb9c7af88582ba0290725` |
| Branch / PR | `main`; commits `92746074431b60f2f96347c940293da10952b292`, `551a35f7c159abc27cf97ac4d0f8bef86a002891`, `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`; no PR |
| Scope interpretation | Repair the reported egui 2→3 regressions, cancel connection tests in the worker, and expose a distinct Cancel Test action while the test is pending. |
| Out of scope | Live provider verification remains pending; exact-height captures are unavailable on this host; no unrelated surfaces or palette redesign. |

## 2. Progress checkpoint

- Current connection-test UI implementation: `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`; prior request cancellation implementation: `551a35f7c159abc27cf97ac4d0f8bef86a002891`; original UI repair: `92746074431b60f2f96347c940293da10952b292`.
- Completed acceptance rows: [x] theme-aware sidebar; [x] modal Tab/Shift+Tab traversal; [x] required/local validation and port-zero rejection; [x] reduced unintended borders; [x] request cancellation through the runtime; [x] a separate localized Cancel Test control appears while pending and its handler leaves the dialog open; [x] worker drops the test future and suppresses completion; [x] provider test resources avoid persistent connector registration.
- Remaining acceptance rows: [ ] live cancellation against each provider; [ ] exact 1440×900 and 1920×1080 captures; [ ] live keyboard interaction; [ ] Save Query As and Agent Settings captures; [ ] independent review and owner verification.
- Findings / risks: original baseline has 0 P0 / 0 P1 / 5 P2, addressed at `92746074431b60f2f96347c940293da10952b292`; cancellation baseline at `1888e812360c2acb59cdb9c7af88582ba0290725` adds one P2, addressed at `551a35f7c159abc27cf97ac4d0f8bef86a002891`. Self-review found 0 introduced P0/P1/P2. Capture shutdown still intermittently aborts in AppKit after writing images.
- Tests already run:
  - `cargo +1.95.0 fmt --all -- --check` → pass, exit 0.
  - `cargo +1.95.0 check --workspace` → pass, exit 0.
  - `cargo +1.95.0 clippy --workspace --all-targets -- -D warnings` → pass, exit 0.
  - `cargo +1.95.0 test --locked -p db-pro-ui --lib` → 1,124 passed / 0 failed / 0 ignored at original UI implementation SHA `92746074431b60f2f96347c940293da10952b292`.
  - `cargo +1.95.0 test --workspace` → 1,793 passed / 0 failed / 41 ignored (22 suites), exit 0, at cancellation implementation SHA `551a35f7c159abc27cf97ac4d0f8bef86a002891`.
  - `cargo +1.95.0 build --release --locked -p db-pro-native` → pass, exit 0.
  - `cargo +1.95.0 build --locked -p db-pro-native --features capture` → pass, exit 0.
  - `cargo +1.95.0 test --locked -p db-pro-ui --lib components::dialog::modal_guard::tests::tab_advances_focus_between_fields_inside_modal -- --nocapture` → 1 passed / 0 failed / 0 ignored, exit 0.
  - `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 12 pass / 4 warning categories / 0 errors, exit 0.
  - `cargo +1.95.0 build --release --locked -p db-pro-native --features capture` → pass, exit 0; two capture-feature dead-code warnings.
  - Release capture smoke at 1280×800 → light-normal exit 0; light-error, dark-normal, dark-error returned `-6` after writing screenshots. See crash report and images below; not a four-state pass.
- Follow-up UI cancellation and failed-dispatch tests, runtime future-drop and stalled-socket tests, native request-ID translation test, and SQLite `test_connection_succeeds` integration test → each 1 passed. Exact commands are recorded in VERIFICATION.md.
- Dependency / blocker changes: no dependency changes. `orca skills get computer-use --json` failed with `command not found: orca`; live keyboard review was unavailable through that route.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` |
| Commit list | `92746074431b60f2f96347c940293da10952b292` original repair; `551a35f7c159abc27cf97ac4d0f8bef86a002891` request cancellation; `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` visible pending-test cancel action |
| File / surface inventory | `crates/native-app/src/capture.rs` — deterministic connection validation capture; `crates/ui/locales/{en,ja,ko,vi,zh}.yml` — localized connection field errors; `crates/ui/src/agent_settings_view.rs` — required API-key label; `crates/ui/src/components/dialog/frame.rs` — remove footer rule; `crates/ui/src/components/dialog/modal_guard.rs` — Tab/Shift+Tab focus containment and regression test; `crates/ui/src/components/dialog/ui.rs` — remove body separator; `crates/ui/src/components/input/layout.rs` — no resting input outline; `crates/ui/src/components/input/search.rs` — preserve intentional search outline; `crates/ui/src/components/input/text.rs` — retain enabled/disabled semantic fills; `crates/ui/src/components/legacy.rs` — theme-aware sidebar buttons; `crates/ui/src/connection/advanced_panels.rs` — required SSH markers/errors and shared port predicate; `crates/ui/src/connection/form_fields.rs` — required connection markers/errors; `crates/ui/src/connection/logic.rs` — reject port 0; `crates/ui/src/connection/tests.rs` — port boundary regressions; `crates/ui/src/connection/view.rs` — SQLite database required/error state; `crates/ui/src/query_dialogs_view.rs` and `crates/ui/src/query_state.rs` — Save Query As error state; `crates/ui/src/query_save_dialog_surface_view.rs` — required marker and inline error; `crates/ui/src/theme.rs` — remove global noninteractive strokes; `crates/ui/src/tokens/component/input.rs` — document input chrome states; `crates/ui/src/workspace_actions.rs` — deterministic invalid connection capture; `docs/plans/STATUS.md`, `docs/plans/active/connection-dialog-accessibility/{PLAN.md,CHECKLIST.md,FINDINGS.md,VERIFICATION.md}`, and `CHANGELOG.md` — lifecycle, evidence, and user-facing change records. |
| Cancellation follow-up file inventory | `crates/ui/src/connection/view.rs`, `runtime_protocol.rs`, connection tests; `crates/native-app/src/translate_cmd.rs` and tests; `crates/runtime/src/worker.rs`; PostgreSQL/SQLite/MySQL/SQL Server connector test-resource paths; CHANGELOG and active plan/evidence files. |
| Acceptance mapping | Theme regression → `legacy.rs:286–307`, `theme.rs:439–447`, light/dark Query captures. Focus regression → `modal_guard.rs:72–94` plus `modal_guard.rs:209–347` test. Required fields/local errors → `connection/form_fields.rs:36–53`, `connection/advanced_panels.rs:232–260`, `query_dialogs_view.rs:220–224`, `query_save_dialog_surface_view.rs:30–38`, `agent_settings_view.rs:119–129`. Port 1–65535 → `connection/logic.rs:8–20` and tests. Border regression → `theme.rs:439–447`, `components/input/layout.rs:23–54`, `components/input/search.rs:153–163`, `components/dialog/frame.rs:27–32`. All source references above are at implementation SHA `92746074431b60f2f96347c940293da10952b292`; baseline findings are at `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`. |
| Commands and counts | Exact commands and exit statuses are listed in §2 and VERIFICATION.md. Full workspace tests: 1,793 passed / 0 failed / 41 ignored. |
| CI run IDs / status | Not run; no PR or CI run was triggered. |
| Known limitations | Host caps logical screenshot height at 838: requested 1440×900 and 1920×1080 captures render at 1440×838 and 1920×838. Live provider cancellation for PostgreSQL, SQLite, MySQL, and SQL Server remains pending; no provider behavior is inferred from the local protocol socket smoke. Real keyboard interaction, independent review, and separate Save Query As / Agent Settings captures remain pending. The latest release matrix wrote all 12 images; 6/12 processes returned SIGABRT after rendering in the AppKit run-loop shutdown path. |
| Migrations / config implications | No schema/config migration or dependency change. Added localized Cancel Test/cancellation status strings and `DB_PRO_CAPTURE_CONNECTION_TEST_PENDING`; no persisted state is affected. |
| Out-of-scope changes | No palette redesign or unrelated UI surfaces; live provider runtime evidence is pending rather than assumed. |

### Runtime capture evidence

The capture binary was run with `DB_PRO_CAPTURE_SETTLE_FRAMES=90`; each subprocess exited 0.

- Connection dialog light/dark, normal/error, requested 1280×800: `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1280x800.png`.
- Connection dialog light/dark, normal/error, requested 1440×900: `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1440x900.png` (effective 1440×838).
- Connection dialog light/dark, normal/error, requested 1920×1080: `/tmp/ui-regression-connection-{normal,error}-{light,dark}-1920x1080.png` (effective 1920×838).
- Query workspace: `/tmp/ui-regression-query-{light,dark}-1280x800.png`.

Connection captures showed visible required markers and inline name/database/port errors; Query light/dark captures showed theme-consistent workspace styling. Save Query As and Agent Settings were not separately captured.

Follow-up release run (2026-10-11), using `target/release/db-pro-native` built with `--features capture`:

- `/tmp/db-pro-release-run-{normal,error}-{light,dark}-1280x800.png` — all four images were written and visually inspected; only light-normal exited 0, the other processes returned `-6`.
- Crash report: `/Users/truongdq/Library/Logs/DiagnosticReports/db-pro-native-2026-10-11-073843.ips`; main-thread stack enters AppKit `NSTouchBarFinderObservation` KVO invalidation during the NSApplication run loop. Capture source at implementation SHA `92746074431b60f2f96347c940293da10952b292` writes the image and requests `ViewportCommand::Close` (`crates/native-app/src/capture.rs:521-525`). This is a localized platform-path observation, not a proven root cause.
 - Current cancellation capture matrix: `/tmp/db-pro-cancel-{normal,error}-{light,dark}-{1280x800,1440x900,1920x1080}.png`; all 12 images were written, six processes exited 0 and six SIGABRT after rendering. Representative light/dark, normal/error images were reviewed. Requested 1440×900 and 1920×1080 were capped to 1440×838 and 1920×838.
 - No in-progress connection-test capture fixture exists in `crates/native-app/src/capture.rs` at source SHA `551a35f7c159abc27cf97ac4d0f8bef86a002891`; loading visuals were not captured. Runtime cancellation is covered by the worker/socket smoke test.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `c02ec5ac9ac42d2c4375f94269f6d773bc642a53` (self-review only; no independent reviewer pass) |
| Verdict | n/a — independent review, live provider verification, and owner interaction remain pending; lifecycle stays RUNTIME_VERIFY |
| P0 / P1 / P2 counts | Original UI baseline: 0 / 0 / 5 P2, addressed. Request-cancellation P2 addressed at `551a35f7c159abc27cf97ac4d0f8bef86a002891`. The missing pending-test Cancel Test affordance was addressed at `c02ec5ac9ac42d2c4375f94269f6d773bc642a53`. Self-review found 0 open P0/P1/P2. Capture shutdown SIGABRTs remain a host-runtime limitation. |
| Findings | The UI and cancellation findings are addressed; source pointers and baseline SHAs are recorded in FINDINGS.md. No open P0/P1/P2 found in self-review. |
| CI disposition | Not run; no CI workflow was triggered. Local pinned tests/build for this follow-up are listed below; earlier workspace gates passed at `551a35f7c159abc27cf97ac4d0f8bef86a002891`. |
| Next task(s) unblocked | Obtain live cancellation evidence independently for all four providers, real keyboard traversal, exact-height capture on a taller display, independent review, and separate Save Query As / Agent Settings captures. |

## 5. Research / audi

… [truncated for memory entry; see source file]

