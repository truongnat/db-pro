# macOS Shutdown Crash (R-01) — Findings

All source anchors are against `0a2418d9a7306ccd526dd428b9e132e64db2e145`.
Upstream anchors are exact SHAs/tags quoted inline.

## P0

None. The defect is not data-destructive and the application is usable; it is a
release blocker (P1) rather than a P0.

## P1

### F-1 — Closing the macOS window aborts the process (release blocker)

- Claim: verified against our own runtime evidence, not inferred.
  `docs/release/evidence/readiness-2026-10-24/observations.json:37` records
  `window_close.process_exit = -6` (`SIGABRT`) with
  `exception = NSRangeException` and
  `detail = "_NSTouchBarFinderObservation removal for nextResponder from WinitView"`.
  The exception text and the AppKit frames are in
  `docs/release/evidence/readiness-2026-10-24/packaged-runtime.log:11` (throw) and
  `:16-20` (frames 5-6, `-[… invalidate]` → `___NSTouchBarFinderSetNeedsUpdateOnMain_block_invoke_2`)
  and `:51` (`libc++abi: terminating due to uncaught exception of type NSException`).
- Cause chain (upstream-authoritative): `accesskit_macos` builds a
  `SubclassingAdapter` per window (it re-classes the `NSView` subclass it observes).
  On drop it restores the original class. `NSTouchBarFinder` registers KVO for
  `nextResponder` against that subclass, so the re-class silently removes the
  registration; the finder's deferred invalidation — which runs from a display-cycle
  observer, one turn *after* the close — then calls
  `removeObserver:forKeyPath:context:` on an observer that is no longer registered,
  AppKit raises `NSRangeException` across QuartzCore's C++ frames, and the process aborts.
  AccessKit/accesskit#835 states this in the maintainers' own words:
  *"macOS can sometime subclass views which then make our subclassing adapter drop
  implementation crash the application. One common cause is with machines that have a
  touch bar. The best we can do is ensure we free the adapter but don't touch the view
  class."* (issue created 2026-10-18, closed by commit `451885c1850ec1800cea1d0660bf8ba62e0f410d`
  committed 2026-10-23, shipped in `accesskit_macos` 0.27.2 published 2026-10-25).
- Why our app is exposed: eframe `accesskit` feature is on in
  `crates/app/Cargo.toml:10` and `crates/ui/Cargo.toml:9`, and `enable_accesskit()`
  runs unconditionally at `crates/native-app/src/main.rs:612`. Our locked graph is
  eframe 0.36.2 → egui-winit 0.36.2 (`accesskit_winit ^0.32.0`) → accesskit_winit
  0.32.2 (`accesskit_macos ^0.26.0`) → **accesskit_macos 0.26.3**, i.e. the last
  version before the fix.

### F-2 — The abort also loses the persisted session state

- `App::save` is the only write path for `dbpro.native.*` keys, and eframe calls it in
  the loop-exit path; an abort during teardown can therefore discard theme, saved
  connections, query documents and query history.
- Observed, not theorised: `observations.json:38` records after restart
  `theme_observed: "light; previous dark selection was not present after the abnormal close"`.
- Consequence for the release: R-01 is also the reason the "restart continuity" claim in
  that evidence file only qualified SQLite connection continuity. Any fix must keep the
  `App::save` + flush cycle running — which is precisely why `std::process::exit(0)` is
  the wrong shape of fix (see Rejected).

### F-3 — Reproduction is hardware-conditional, so CI as configured cannot prove it

- `ci.yml:16` is `runs-on: ubuntu-latest`; macOS is built only by `release.yml`
  (`macos-14`, `release.yml:305`, `:443`), which runs on `workflow_dispatch`/tag.
  A macOS-only regression therefore has no compile gate before release time, let alone a
  runtime gate.
- Touch Bar Finder only exists where a Touch Bar (or `TouchBarServer`) is present:
  zed-industries/zed#64819 is "Not reproducible on our non-Touch Bar development
  machines (Mac16,1, arm64)", and the follow-up comment notes it also fires on a Mac
  without a Touch Bar while `TouchBarServer` runs (iPad sidecar / the DFRFoundation
  simulator). Our own crashing host was `macOS 25.6.0 arm64`.
- Additional discriminator recorded upstream: closing through Accessibility (`AXPress`)
  does **not** reproduce it; a real mouse click on the red button does
  (Bomret, storytold/craft-libs#3). Automated a11y-driven close tests will pass on a
  broken build.

### F-4 — The crash already corrupts our own evidence pipeline

Long-standing, not new to the 2026-10-24 run. Two earlier rows in `docs/plans/STATUS.md`
(at `0a2418d9`) record the same abort from the deterministic capture harness:

- `STATUS.md:196` (Native UI Regressions after egui 2→3): "6/12 latest release capture
  processes ended in AppKit NSTouchBar SIGABRT after rendering."
- `STATUS.md:135` (Viewport Select, Complex Picker and Light Chrome Fit): "1440/1920 render
  at 838 logical height, `exit=-6` cells are the post-capture macOS TouchBar crash."

Consequence: `AGENTS.md`'s mandatory runtime-capture gate is unreliable on macOS — a capture
that renders its PNG and then aborts reports a non-zero exit, which is how "exit=-6 cells"
entered those capture matrices. The 838 logical-height cap noted in the same two rows is a
*macOS host display* constraint and does not reproduce on Linux/Xvfb. Evidence is the surviving
in-repo artifacts, measured from PNG IHDR headers on 2026-10-26 (665 PNGs scanned):
the hidden-Xvfb matrix in `docs/plans/active/qa-keyboard-correctness-wave/captures/` holds
**29 files at exactly 1440×900 and 13 at exactly 1920×1080**, while the macOS-host matrices hold
**154 files at 2880×1676 (= 1440×838 @2x) and 135 at 3840×1676 (= 1920×838 @2x)**, many named
`*-host-capped*` / `*-clamped*`. The Xvfb setup is recorded at
`docs/plans/active/qa-keyboard-correctness-wave/VERIFICATION.md:50-52`. A *fresh* capture re-run
at HEAD on this Linux host did **not** complete (both sizes exit 124 after 120 s, no PNG; the app
re-attached the saved PostgreSQL connection and introspected 1,108 tables, so the settle loop never
finished) — the conclusion rests on the retained artifacts, not on that run. The exact macOS
mechanism (menu bar vs Dock vs screen bounds) is still unproven, and 900→838 / 1080→838 is not a
constant delta, so the two symptoms must not be conflated in the release docs but neither may the
cap be attributed to application code.

## P2

- The in-app window control at `crates/app/src/shell/chrome/shell_topbar_view.rs:299`
  sends `ViewportCommand::Close`, i.e. it reaches the same teardown as the native
  button, and the capture driver does the same at `crates/native-app/src/capture.rs:595`.
  Not separately measured on macOS; treat both as in-scope for the acceptance protocol.
- `Cmd+Q` is the one path that is clean upstream (via
  `PredefinedMenuItem::quit` → `-[NSApplication terminate:]`, `crates/native-app/src/app_menu.rs:54`),
  which is both a user workaround and the strongest hint for M2's shape.

## Upstream fix reachability

Verified against the crates.io sparse index (dependency ranges quoted verbatim):

| Crate / version | Requires | Reaches the fix? |
|---|---|---|
| egui-winit 0.36.2 (pinned by eframe 0.36.2) | `accesskit_winit ^0.32.0` | no |
| accesskit_winit 0.32.2 | `accesskit_macos ^0.26.0` | no (max 0.26.x) |
| accesskit_winit 0.33.0 | `accesskit_macos ^0.26.1` | no |
| accesskit_winit 0.34.2 | `accesskit_macos ^0.27.2` | yes |
| egui-winit 0.37.0 | `accesskit_winit ^0.34.0` | yes |

So the durable fix requires the eframe 0.36 → 0.37 upgrade wave; no `cargo update`
on the current manifest can reach `accesskit_macos` 0.27.2.

## Hardware evidence for the mitigations

- M2 (`-[NSApp terminate:]` on close): storytold/craft-libs#3 — same stack
  (eframe 0.36.2, winit 0.30.13, accesskit_macos 0.26.3). Baseline 7/7 runs crashed on
  the red button; with `craft_quit::on_close_request(ctx)` 0/11 crashed, all exit status
  0, and `on_exit`/state persistence ran every time. `CancelClose` (unsaved-changes
  prompt) is respected by returning early when a `CancelClose` is already queued.
  Reproduced 5/5 on CADCraft, FilmCraft, SoundCraft, ArtCraft Launcher and confirmed on
  WordCraft/DeckCraft/DesignCraft (exit 133/134).
- M4 (accessibility off): storytold/designcraft#831 reported 0 crashes in 24 launches
  with eframe's `accessibility` feature off — and the same report says a launch-time
  variant exists (~1 in 4 launches), which `craft-quit`'s close path does not cover.
  The team refused the change because it removes VoiceOver support.
- M5 (forced exit): vectorcraft#670 exits the process in `on_exit` before teardown; the
  author recorded that it "was not tested on a Touch Bar Mac", so it is unproven for the
  crash case even where it was adopted.

## Rejected / downgraded findings

| Original claim | Evidence checked | Decision |
|---|---|---|
| Gap doc § 1.4: "Nguyên nhân gốc và phương án khắc phục đang chờ tái hiện và điều tra sâu ở tầng winit/AppKit" — i.e. cause unknown | F-1 chain, AccessKit#835 commit `451885c`, craft-libs#3 hardware logs | Cause is now established upstream; keep the item open only for the *verified fix*. Doc updated. |
| "Disable AccessKit to stop the crash" (raised as the cheapest fix) | designcraft#831 measurement plus our own a11y contract (`app_lifecycle_tests.rs` asserts `accesskit_update` is present) | Rejected: ships a screen-reader-free product and breaks repo accessibility gates. |
| "Wrap quit in `std::process::exit(0)`" | Release contract in the gap doc § 3; F-2 shows state loss is half the damage | Rejected: prohibited and counter-productive. |
| "Add a wall-clock grace period between close and release" | zed#64819: a carried `WINDOW_TEARDOWN_GRACE_PERIOD = 100ms` did **not** stop identical crashes | Rejected: the wait must be one real display cycle, not a timer; idle apps may not run one. |
| "The crash is in DB Pro code" | `packaged-runtime.log:16-20` — frames 5-13 are AppKit/QuartzCore/Foundation; the Rust frames are unsymbolised release addresses inside `db-pro-native`'s `-[NSApplication run]` loop | Downgraded as a claim: our code is not on the throwing stack; our *dependency graph* is the cause. |

## Runtime gaps

- PostgreSQL: n/a — no provider path involved.
- SQLite: n/a for the crash; F-2 (lost persisted state) needs re-measurement on macOS
  after any fix, because that is the observable symptom of the abort.
- UI: BLOCKED — no reproducible close/crash cycle on this Linux host. Both the
  pre-fix baseline and the post-fix confirmation must come from a macOS host, with the
  `~/Library/Logs/DiagnosticReports` listing captured either way.
