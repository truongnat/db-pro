---
schema_version: 1
id: "MEM-SRC-e51a0c1fef6b4eda"
title: "Source mirror: docs/plans/active/connection-dialog-accessibility/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/connection-dialog-accessibility/VERIFICATION.md"
source_sha256: "e51a0c1fef6b4edafbcc0ed9c64ec7874592a6764cb72fa25dace2e704d69d29"
mirror_body_sha256: "e51a0c1fef6b4edafbcc0ed9c64ec7874592a6764cb72fa25dace2e704d69d29"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
