# macOS Shutdown Crash (R-01) — Plan

State: BLOCKED
Branch: `fix/macos-shutdown-crash` (work recorded on `main` at `0a2418d9a7306ccd526dd428b9e132e64db2e145`)

Blocked on: a macOS host — specifically a machine where AppKit's `NSTouchBarFinder`
exists (a Mac with a Touch Bar, or any Mac running `TouchBarServer`). The throwing
frame is inside AppKit; the defect is not reachable from Linux/Xvfb and cannot be
compile-checked here either, because `ci.yml:16` runs `ubuntu-latest` only.

## Goal

On macOS, closing the DB Pro window (native red button, Cmd+W, or the in-app window
control) must terminate the process with status 0 **and** complete eframe's storage
cycle, so that the state written by `App::save` (theme, saved connections, query
documents, query history, SSH profiles) survives the quit.

## Evidence / baseline

- Current implementation: `crates/native-app/src/main.rs:573` keeps native window
  decorations on macOS (`with_decorations(!cfg!(target_os = "linux"))`), `main.rs:612` calls
  `creation_context.egui_ctx.enable_accesskit()` on every launch, and
  `crates/app/Cargo.toml:10` plus `crates/ui/Cargo.toml:9` both enable eframe's
  `accesskit` feature.
- Known failure scenario: the packaged release smoke run on macOS 25.6.0 arm64 closed the
  window and the process aborted — `docs/release/evidence/readiness-2026-10-24/observations.json:37`
  (`process_exit: -6`, `SIGABRT`, `NSRangeException`, `_NSTouchBarFinderObservation` /
  `nextResponder` / `WinitView`). The full AppKit stack is in
  `docs/release/evidence/readiness-2026-10-24/packaged-runtime.log:11-51`.
  The same record (`observations.json:38`) shows the persistence cycle did **not**
  survive it: the dark theme selected before the quit was gone after restart.
- Existing tests: none cover quit/close behaviour. `crates/app/src/app/tests/app_lifecycle_tests.rs`
  drives frames through `test_frame::frame` and never exercises a window close.

## Root cause (established, see FINDINGS.md)

`accesskit_macos`'s `SubclassingAdapter` re-classes the `WinitView` back to its
superclass when the adapter drops. AppKit's `NSTouchBarFinder` registers KVO on
`nextResponder` against that subclass, so re-classing silently removes the
registration; in the next display-cycle flush `-_NSTouchBarFinderObservation invalidate`
calls `removeObserver:forKeyPath:` on an observer that is no longer registered,
AppKit raises `NSRangeException` on a C++ frame, and the process aborts. No DB Pro
frame is on the throwing stack.

DB Pro pins the exact affected combination: eframe 0.36.2 → egui-winit 0.36.2 →
`accesskit_winit ^0.32` → `accesskit_macos ^0.26.0`, locked at 0.26.3
(`Cargo.lock`, `crates/native-app/Cargo.toml:24`).

## Scope

- Record the confirmed cause, the dependency chain that makes the upstream fix
  unreachable from the pinned stack, and the ranked remediation options with the
  hardware evidence behind each (this document set).
- Fix the v0.1.0 release gap documentation so R-01 is described by its real cause
  instead of "nguyên nhân gốc đang chờ điều tra".
- Define the acceptance protocol for the macOS host that lands the fix.

## Non-goals

- Performing the eframe 0.36 → 0.37 upgrade inside this feature (separate wave, own plan).
- Disabling AccessKit / the eframe `accesskit` feature. That removes VoiceOver support for
  every user and breaks the accessibility contract the repo tests enforce.
- Any `std::process::exit(0)` shortcut — explicitly prohibited by
  `docs/release/2026-10-25-v0.1.0-audit-and-release-gap.md` § 3, and it is also wrong:
  it skips eframe's `App::save` + storage flush, which is half of the damage R-01 causes.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | n/a | Not a provider-facing change; no SQL path involved. |
| SQLite | n/a | Connection persistence is only an observer of the bug (state lost on abort). |

## Architecture

Close paths that reach AppKit teardown, all owned by `db-pro-native` / `db-pro-app`:

```text
native red button / Cmd+W  → winit windowShouldClose → eframe exits the loop
                                                    → App::save → storage flush
in-app window control      → TaskBridge/UI: ViewportCommand::Close (crates/app/src/shell/chrome/shell_topbar_view.rs:299)
capture driver             → ViewportCommand::Close       (crates/native-app/src/capture.rs:595)
Cmd+Q (native menu quit)   → -[NSApplication terminate:]  (crates/native-app/src/app_menu.rs:54)
```

The abort happens after eframe's own teardown completes — inside the AppKit run loop,
one display cycle later — which is why the crash is timing- and hardware-dependent and
why a clean exit must keep the native view alive until AppKit retracts its observations
(it cannot be fixed by adding a Rust-side wait).

## Remediation options

| ID | Option | Durable | Verifiable from this host | Evidence |
|---|---|---|---|---|
| M1 | Upgrade eframe 0.36.2 → 0.37 so `accesskit_winit ^0.34` pulls `accesskit_macos ^0.27.2`, which contains upstream fix `451885c` "Fully drop macOS SubclassingAdapter if KVO is detected" (AccessKit/accesskit#835, released 2026-10-25) | yes | lockfile chain yes, runtime no | FINDINGS § Upstream fix |
| M2 | Intercept the close request on macOS and terminate through `-[NSApplication terminate:]` instead of letting eframe destroy the window (the `craft-quit` approach) | mitigation | no (macOS-gated ObjC, Linux-only CI) | FINDINGS § Hardware evidence, 7/7 → 0/11 |
| M3 | `[patch.crates-io]` a fork of `accesskit_macos` 0.26.x with `451885c` cherry-picked | mitigation | no | same commit, plus fork maintenance cost |
| M4 | Drop the eframe `accesskit` feature on macOS | rejected | — | kills VoiceOver; DesignCraft explicitly refused it |
| M5 | Force-exit the process from the exit hook | rejected | — | prohibited by the release contract and loses `App::save` |

Recommended: **M1** (the only option that removes the defect at its source and keeps
accessibility intact), with **M2** as the ship-now fallback if the 0.37 wave cannot land
before the v0.1.0 macOS artifact is produced. Either way the macOS artifact stays gated
until the acceptance protocol below passes on hardware.

## Acceptance criteria

- [ ] Red close button, Cmd+W, the in-app window control, and capture-mode `Close` each exit
      with status 0 and no `NSRangeException` / no report in `~/Library/Logs/DiagnosticReports`.
- [ ] `App::save` output is present after the quit: theme, saved connection ids, and
      `dbpro.native.query-history-v1` survive a restart.
- [ ] VoiceOver still exposes the shell (accessibility not disabled as a side effect).
- [ ] `cargo build --release --locked -p db-pro-native` green on macOS arm64 and the
      release workflow produces the macOS artifact from the fixed SHA.
- [ ] Evidence recorded in VERIFICATION.md with the exact SHA and the crash-report listing.
