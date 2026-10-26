# macOS Shutdown Crash (R-01) — Checklist

## Planning
- [x] Evidence and failure scenario recorded (FINDINGS F-1: our own
      `packaged-runtime.log:11,16-20,51` + `observations.json:37`; cause chain from
      AccessKit/accesskit#835 commit `451885c`)
- [x] Scope/non-goals explicit (PLAN.md; 0.37 upgrade, AccessKit removal and forced exit
      are all declared out of scope)
- [x] PostgreSQL/SQLite support matrix explicit (n/a — documented as such in PLAN.md and
      FINDINGS "Runtime gaps")
- [x] Remediation options ranked with the evidence and the reachability of each
      (PLAN.md options table + FINDINGS "Upstream fix reachability")

## Implementation
- [ ] Fix selected and landed on a macOS host: M1 (eframe 0.37 upgrade, preferred) or
      M2 (close → `-[NSApp terminate:]`, ship-now mitigation) or M3 (patched
      `accesskit_macos`)
- [ ] macOS compile gate in place before landing macOS-gated code — `ci.yml` currently has
      no non-Linux job, so M2/M3 code would otherwise first be compiled at release time
- [ ] Safety/capability rules respected — no `std::process::exit(0)`, AccessKit stays on
- [ ] Cache/state invalidation accounted for: verify `App::save` + storage flush still run
      on the fixed quit path (FINDINGS F-2)

## Tests
- [ ] Regression test added for the quit invariant once a fix exists. Shape: on macOS, a
      close request must not reach eframe window teardown while AppKit can still observe
      the view; the test must assert exit status 0 *and* persisted keys present
      (`dbpro.native.dark-mode`, `dbpro.native.query-history-v1`, saved connection ids).
- [x] PostgreSQL evidence recorded or explicitly pending/N/A (N/A)
- [x] SQLite evidence recorded or explicitly pending/N/A (N/A for the crash; F-2
      persistence re-measurement recorded as pending macOS)

## Review
- [ ] Architecture review complete (owns: `crates/native-app` launch path, `crates/app`
      storage cycle, and the eframe/AccessKit dependency chain)
- [ ] Correctness/security review complete
- [ ] P0 = 0 (holds: no P0 recorded)
- [ ] P1 = 0 (open: F-1 and F-2)

## Runtime
- [ ] macOS close-cycle matrix on hardware: red button, Cmd+W, in-app window control
      (`shell_topbar_view.rs:299`), capture-mode `Close` (`capture.rs:595`), Cmd+Q
      — each must exit 0 with no DiagnosticReports entry
- [ ] Real mouse-event driven, not `AXPress` (FINDINGS F-3)
- [ ] VERIFICATION.md contains actual commands/evidence
- [ ] STATUS.md matches state/folder
