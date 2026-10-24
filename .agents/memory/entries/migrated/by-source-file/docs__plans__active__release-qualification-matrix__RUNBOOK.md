---
schema_version: 1
id: "MEM-SRC-3f5acd34076c978a"
title: "Source mirror: docs/plans/active/release-qualification-matrix/RUNBOOK.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/RUNBOOK.md"
source_sha256: "3f5acd34076c978a26f68d7d861c70db7476512b744b776e1cf537337b3a0f14"
mirror_body_sha256: "3f5acd34076c978a26f68d7d861c70db7476512b744b776e1cf537337b3a0f14"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Runbook — Orca qualification execution

**Chưa được thực thi bởi plan này.** Start only after owner authorizes execution. Product baseline used to design cases: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`; each actual run freezes its own candidate SHA/artifact. Read PLAN, REQUIREMENTS, FIXTURES, assigned CASES and RESULT.schema before acting.

## A. Intake và immutable run manifest — L0 owns

1. Freeze `run_id`, intended channel (internal/external-RC/public), source full SHA + dirty inventory, compiler pin and **actual** version, lockfile hash, exact executable/archive hashes/version/features, target platform/OS/architecture and provider capability snapshot. Pin compiler explicitly; ambient `RUSTUP_TOOLCHAIN=stable` must not override release qualification.
2. Product code/assets must match candidate. Planning/report-only dirty files listed separately; uncommitted product changes require a different candidate qualification and owner disposition. Old/source-only/test-only/capture evidence cannot replace exact artifact outcome.
3. Default full support matrix retains four selectable providers and three configured platform targets. Lead expands CASES.json into candidate instances **before workers run**, writes immutable `expected-instances.json` with digest, assignment map, approved N_A/capability exclusions and conditional scope decisions. No case deletion because unavailable host/fixture. See AGGREGATION.md.
4. Artifact owner performs build/package once, outside GUI workers. No concurrent cargo builds swapping release/capture binaries. Keep release untouched; capture/debug artifacts have different digest and `role`, only supplemental.
5. Fixture owner provisions manifest-owned sandbox per FIXTURES. User plan approval alone is not approval to send AI data/spend, install certificates, change OS trust/global SSH/Keychain, mutate business DB, publish/release/tag/push, create OS accounts or delete Docker volumes.
6. Record prerequisites per platform. Host or Orca capability unavailable => BLOCKED instances, not inferred PASS from macOS. Windows/Linux cannot be claimed via cross compilation.
7. Verify actual UI storage isolation in addition to runtime data dir. OS sandbox standard; if feature graph/storage indicates persistence absent, **execute persistence acceptance safely and record actual FAIL when observable**, not edit app or disable requirement. Environment inability to isolate is BLOCKED before touching user state.

## B. One GUI lease per desktop — no exceptions

L0 manages `runs/<run_id>/desktop-leases/<desktop_id>.json`: owner worker, epoch, platform/profile, app PID/window, acquired/released time, cleanup disposition. This is an orchestration contract, **not a lock implementation already shipped**.

- At most one active lease per real desktop, including separate app windows; source-read/CLI workers may parallelize only on independent files/fixtures and must not steal GUI focus or mutate that GUI case's data.
- Sequence: lead grants lease → worker acknowledges exact epoch/PID/window → baseline oracle → case actions → evidence/oracles → cleanup → worker release → lead verifies safe handoff → next worker.
- Independent databases do **not** permit two simultaneous Orca GUI workers on one desktop. Parallel GUI requires separately registered OS desktops/VMs/hosts and isolated app/storage/keyring/fixtures.
- Lease timeout/worker crash is not automatic permission to act. Lead checks owned process/controller state and collects partial evidence; revoke old epoch; clean up owned resources or mark contaminated/BLOCKED. Never kill another worker/user process by name.
- Second DB writer for conflict belongs to current worker or fixture controller with explicit before-save/after-commit handshake; no unscheduled mutation during another case.

## C. Start and select the exact app

1. Resolve Orca executable by installed `orca-cli`/`computer-use` skill; load version-matched guides first. On non-Orca Linux do not run bare GNOME `orca` screen reader.
2. Check permissions/capabilities; list apps/windows read-only; start only own extracted release with run data/profile. Prefer supervisor that captures stdout/stderr and real exit status. `open` launch and direct executable launch are separate acceptance paths; process-alive is not visible-window proof.
3. Bind `pid:<owned_pid>` plus window-id/index from fresh list. Never select only app name when user/debug/release processes coexist. Check executable hash/PID before every case or restart.
4. Obtain fresh AX tree **and** screenshot; inspect pixels. Persist copies immediately under run evidence; temporary Orca screenshot path/expiry alone is not durable evidence.
5. Clean release uses no capture feature/env hooks. Deterministic capture may supplement specific visual analysis under distinct artifact role but cannot close native action/DB/quit/Keychain tests.

## D. Each case action loop

1. Read assigned definition and all independent oracles; validate prerequisites, fixture marker, server identity and outside-namespace sentinels. Record before screenshot/catalog/data digest. Existing fixture/source count is not actual oracle until queried.
2. Navigate via fresh semantic labels from real tree. Actual anchors mapped in DISPATCH. Indexes expire after navigation/focus/re-render; never hardcode from a previous case or another platform.
3. Perform one intent; take returned/fresh state, then verify actual effect. Tool success/AXPress `unverified`/AXSetValue `readback_unsupported` does not imply UI value/action succeeded.
4. SQL entry: inspect actual rendered buffer/text after input, then run and check distinguishable result oracle. If AXSetValue did not replace buffer, do not run seeded SQL and claim custom scalar PASS. Native real input path needs owned-window OS focus; otherwise BLOCKED input scenario.
5. Use real pointer for outside-click, hover/cursor, drag/pan/resize/context menus. AXPress is semantic activation, not pointer modality. Keyboard-only case uses **real keyboard**, not AXPress/coordinates. Before global input, verify frontmost/focused OS window is owned target; never send unchecked global keystrokes to recover from focus failure.
6. Coordinates: Orca coordinates window-local per live guide. For authorized OS-level fallback, convert fresh screenshot by actual scale **and** window origin to global coordinates; don't assume retina=2. Save original and restore clipboard on same owned session when using copy/paste. If keyboard layout blocks cliclick typing, do not claim app input failure or send unverified global fallback.
7. For async work wait for observed state transitions, request completion and independent DB oracle under declared operation deadline. Fixed sleep or queued/scheduled toast alone is not completion proof. Record delayed/cancelled/stale transitions without unsafe retry.
8. Native DB case must prove app intent → command/runtime → provider effect → event/refreshed UI. CLI oracle alone, synthetic/transfer harness, image of static capture state or code inspection is not native lifecycle evidence.
9. Every expected oracle becomes check `oracle-01..N`; add explicit subchecks for each listed boundary/action. Parent instance PASS only when **all** required checks performed and passed. Missing one boundary = NOT_TESTED/BLOCKED parent, not narrow PASS.
10. Persist `RESULT.schema.json` compliant single attempt record plus artifacts. Scrub secrets, retain synthetic values only when not secret; metadata hash after redaction, raw secrets never committed.

## E. Status rules

| Status | Rule |
|---|---|
| PASS | All requested assertions actually observed, independent oracle matches, correct artifact/variant, evidence opened/retained, safe cleanup complete. |
| FAIL | Actual executed product behavior differs from requirement on valid fixture; exact expected/actual/repro/evidence. Severity independent of test priority. |
| BLOCKED | Missing host/permission/fixture/isolation/input ability/measurement method/authorization, or exact requested geometry cannot be achieved by host. Name failed prerequisite and impact. |
| NOT_TESTED | No attempt for this instance, cancelled scheduling or remaining work. Never convert to PASS or silently remove. |
| N_A | Explicit lead-approved product/platform capability exclusion with source/capability evidence and applicable negative-gating test. No fixture/host is not N_A. |

Unsupported positive feature can be N_A, but CAP-01 negative gate still required. Partial work is check-level evidence and parent BLOCKED/NOT_TESTED/FAIL as appropriate; there is no vague parent PARTIAL-as-PASS status. Existing known failures stay linked to historical artifacts; do not rerun unchanged user-reported failures just to confirm them. A new-candidate regression attempt or authorized qualification is separately traceable, never treated as historical failure disappearing by test omission.

Failure attribution: tool/environment failure ≠ product failure. Example: lost AX after close → inspect process exit and stderr; `SIGABRT/NSRangeException` on the owned release process is product/runtime evidence even if click API returned AX error. Don't call it capture-only when binary is non-capture.

## F. Visual matrix

- CASES JSON has13 named visual definitions; each expands declared surfaces/states, Light/Dark and content1280×800,1440×900,1920×1080. Functional definitions stay at Dark/1280×800, not multiplied by all6 geometry cells.
- Capture **actual content size**, native window size, screenshot pixel dimensions and scale separately. macOS native titlebar adds size; prior audit window1280×832 produced2560×1664 image for1280×800 requested content. Never subtract guessed titlebar height on another OS.
- Host-clamped1440×838/1920×838 is useful observed evidence but **BLOCKED for exact900/1080 geometry**. Do not resize/pad image or rename filename to turn it into requested viewport proof. Achieve full height on other authorized display/VM if available, else report gap.
- Live state recipes are in each visual case. If a named sub-surface contract genuinely has no loading/empty state (e.g. static Hub), lead approves per-instance N_A before results freeze, backed source; do not fabricate that state or confuse missing delay fixture with no state contract. No generic screenshot reused across surface IDs.
- Inspect clipped text/buttons, active tab close visibility, selector/dialog layout, modal layering, empty/error/loading semantics, semantic theme/contrast and keyboard focus/cursor. Keep hashes + worker-inspected flag; image existence/hash only is insufficient.
- Palette/Settings typography/contrast is evaluated against frozen design tokens/rendered output, not web/mobile/ARIA rules. Native AX/screen reader proof remains separate.

## G. Measurement contract

Proposed acceptance budgets for this plan (to be frozen with approval, no retrospective tuning): idle after10s warmup sampled for60s, mean app CPU ≤2% of **one logical core**, RSS growth ≤20MiB over sample; interactive input-to-visible response p95≤100ms across20 measured actions while a bounded long query runs. Record sampling/clock/CPU normalization per OS and raw observations; server jobs and hardware noted. If approved budget differs, manifest must record owner decision before measurement.

ER reference from project perf-audit guidance:200/500/1000 tables time-to-interactive<2/5/10s, layout<0.5/1.5/3s, pan/zoom frame avg<8/12/16ms and p95<16/24/33ms. These are measurement requirements, not prior PASS. No usable instrumentation => BLOCKED, not subjective “smooth”. Criterion data, debug frame measurements and release UI timings are separate evidence classes. No benchmark execution while source/build replacement or another resource-heavy qualification contaminates run.

## H. Safe failure and cleanup

- Stop mutation sequence on unexpected target, changed outside sentinel, unexplained partial write or suspected P0/P1. Preserve logs/preimage; no automatic retry/rollback of user objects. Notify lead; rest of contaminated fixture BLOCKED until owner reset.
- No source edits, feature toggles to bypass checks, assertion suppression, changing capabilities or replacing failing paths with mock during QA. Fixes are separate implementation tasks/candidate SHA, then affected regression and smoke set rerun.
- On crash collect owned process exit, stderr/OS crash report, fixture digest; restart only for assigned recovery scenario. Do not kill/restart Orca/global AX or modify system permissions without separate approval; preserve partial evidence.
- Normal Quit verification is not SIGTERM; emergency owned-process SIGTERM cleanup recorded as cleanup, never clean-exit PASS.
- Restore only case-owned DB/file/keyring entries; verify outside sentinels and no orphan own tunnel/controller. Retain known failing snapshots and artifacts until disposition; no deleting evidence as cleanup.
- Worker writes canonical AGENT_EVIDENCE sections1–6 in its lane run folder with actual source/artifact, statuses, commands, constraints and Vietnamese summary; release GUI lease explicitly. Lead issues aggregate only after reconciling all assigned/missing instances.
