# macOS Shutdown Crash (R-01) — Verification

State: BLOCKED (external dependency: macOS host with a Touch Bar / TouchBarServer)

## Commands actually executed

All on this Linux host at `0a2418d9a7306ccd526dd428b9e132e64db2e145`. Nothing here is
runtime evidence of the crash or of a fix — see "Remaining evidence gaps".

```text
.agents/scripts/search-memory.sh "macOS shutdown crash touchbar"
.agents/scripts/search-memory.sh "SIGABRT close window eframe"
→ "Không có kết quả." for both; no prior corpus entry for this defect.
```

```text
grep -A3 'name = "winit"' Cargo.lock; grep -A3 'name = "eframe"' Cargo.lock;
grep -A3 'name = "objc2"' Cargo.lock
→ winit 0.30.13, eframe 0.36.2, objc2 0.5.2 + 0.6.4 (matching the upstream reports' stack)
```

```text
grep -B1 -A3 'name = "accesskit' Cargo.lock
→ accesskit_macos 0.26.3, accesskit_winit 0.32.2, accesskit 0.24.1 + 0.25.1,
  accesskit_unix 0.21.1, accesskit_windows 0.32.1
```

```text
cargo tree -i -p accesskit_macos --depth 3
→ "warning: nothing to print. To find dependencies that require specific target
   platforms, try to use option `--target all` first" — the macOS adapter is
   target-gated, so reverse-deps were read from Cargo.lock and the crate manifests
   instead of cargo tree on this host.
```

```text
sed -n '88,104p' ~/.cargo/registry/src/*/egui-winit-0.36.2/Cargo.toml
→ accesskit_winit = "0.32.0" (optional)   # i.e. ^0.32.0 — cannot reach 0.34
```

```text
cargo search accesskit_macos --limit 1; cargo search accesskit_winit --limit 1;
cargo search eframe --limit 1; cargo search winit --limit 1
→ accesskit_macos 0.27.2, accesskit_winit 0.34.2, eframe 0.37.0, winit 0.31.0-beta.4
```

```text
curl -s https://index.crates.io/ac/ce/accesskit_winit | (parse deps)
curl -s https://index.crates.io/eg/ui/egui-winit  | (parse deps)
→ accesskit_winit 0.32.2 → accesskit_macos ^0.26.0
  accesskit_winit 0.33.0 → accesskit_macos ^0.26.1
  accesskit_winit 0.34.0 → accesskit_macos ^0.27.0
  accesskit_winit 0.34.2 → accesskit_macos ^0.27.2
  egui-winit 0.36.2 → accesskit_winit ^0.32.0
  egui-winit 0.37.0 → accesskit_winit ^0.34.0
  ⇒ the fix is unreachable without eframe 0.37
```

```text
gh api repos/AccessKit/accesskit/contents/adapters/macos/CHANGELOG.md
→ ## [0.27.2] (2026-10-25) Bug Fixes: "Fully drop macOS SubclassingAdapter if KVO is
   detected (#835)"
```

```text
gh api repos/AccessKit/accesskit/commits/451885c1850ec1800cea1d0660bf8ba62e0f410d
gh api repos/AccessKit/accesskit/issues/835
→ committed 2026-10-23T15:28:57Z; adapters/macos/src/subclass.rs +122/−45 plus
  ~239 lines of main-thread integration tests; issue body quotes the Touch Bar cause
  verbatim (reproduced in FINDINGS F-1)
```

```text
gh api repos/storytold/craft-libs/issues/3 (+comments)
gh api repos/zed-industries/zed/issues/64819 (+comments)
→ hardware evidence quoted in FINDINGS "Hardware evidence for the mitigations"
   (7/7 baseline crash → 0/11 with the terminate: fix, exit 0, on_exit ran)
```

```text
search pattern `runs-on|macos|matrix` over .github/workflows/ci.yml, release.yml
→ ci.yml:16 runs-on: ubuntu-latest (no macOS job); release.yml:305 and :443 use macos-14
```

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | N/A | N/A | No provider-facing change; crash is in window teardown. |
| SQLite | N/A | NOT VERIFIED | Persistence-after-quit (F-2) is only measurable on macOS. |

## UI lifecycle

```text
red close button / Cmd+W / ViewportCommand::Close
→ winit windowShouldClose
→ eframe loop exit (+ App::save, storage flush)
→ NSWindow/WinitView teardown, AccessKit SubclassingAdapter drop (re-class)
→ [one display cycle later] NSTouchBarFinder invalidate → removeObserver → NSRangeException
→ abort (-6)   ← observed on macOS 25.6.0 arm64, packaged-runtime.log:11,51
```

Status: NOT VERIFIED on this host (BLOCKED). The only leg with direct DB Pro evidence is
the last arrow, from the 2026-10-24 readiness run.

## Remaining evidence gaps

- No repro on Linux/Xvfb: the throwing frames are AppKit-internal; the crash needs a
  Touch Bar or a running `TouchBarServer`.
- No baseline re-measure of exit status and DiagnosticReports on a current macOS build.
- No compile check of any macOS-gated fix code: `cargo check` on this host does not
  evaluate `#[cfg(target_os = "macos")]` blocks, and `ci.yml` has no macOS job. Landing
  M2/M3 therefore needs either a macOS CI job or a macOS host run before merge.
- Whether the launch-time variant (storytold/designcraft#831, ~1 in 4 launches) also
  affects DB Pro is unknown; a close-only fix would not cover it.
- F-2's persisted-state loss must be re-measured after the fix (theme +
  `dbpro.native.query-history-v1` + saved connection ids surviving a red-button quit).

## Artifact retention

No new artifacts produced by this investigation. Evidence referenced here is the existing
`docs/release/evidence/readiness-2026-10-24/` set (`packaged-runtime.log`,
`observations.json`), already covered by `docs/ARTIFACT_POLICY.md`. A macOS qualification
run must drop its crash-report listings and exit-status transcript into
`docs/plans/active/macos-shutdown-crash/runs/<date>/` with the `runs.gitignore` template.

## Completion decision

Not COMPLETED and not REVIEW. The plan stays `BLOCKED` under FEATURE_LIFECYCLE.md because
an external dependency (macOS Touch-Bar-class hardware, plus a compile gate we do not have
on Linux) stands between the established root cause and a verified fix. R-01 must remain a
release blocker until a host run satisfies the acceptance criteria in PLAN.md.
