# Native UI Regression Repairs — Verification

State: RUNTIME_VERIFY  
Baseline source SHA: `c6a033da39dc1c1009201fe4c102a05fe17e3dfb`; all implementation edits remained
uncommitted in the worktree during verification.

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

## UI lifecycle

```text
egui theme application → shared theme-aware component styling
modal Tab / Shift+Tab → in-modal input focus chain → containment on next pass
connection/query form submit → existing validation → local field feedback
```

Connection normal/error states were exercised in the native capture runtime. Save Query As and
Agent Settings were verified through source, compilation, and shared input component behavior, but
did not receive separate surface captures. PostgreSQL and SQLite behavior was not exercised because
no provider path changed.

## Review

Self-review complete: no P0/P1 findings; P2 presentation/accessibility regressions are addressed in
the current worktree. Independent review and owner keyboard interaction remain pending. The
computer-use skill selected `orca`, but `orca skills get computer-use --json` returned
`command not found: orca`; live keyboard interaction was not available through that route.

## Remaining evidence gaps

- Capture at exact 1440×900 and 1920×1080; the current display caps height at 838 logical pixels.
- Verify real Tab/Shift+Tab interaction in a launched native window.
- Capture Save Query As and Agent Settings surfaces independently.
- Obtain independent review and owner verification.

## Completion decision

Remain RUNTIME_VERIFY. Pinned workspace gates pass and available native captures were reviewed,
but exact-height captures, live keyboard interaction, the remaining two form captures, and
independent review are still open. Do not mark COMPLETED until all applicable lifecycle gates are
satisfied.
