---
schema_version: 1
id: "MEM-SRC-56690b74b51bfa38"
title: "Source mirror: docs/plans/active/native-ui-focus-consistency/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/native-ui-focus-consistency/FINDINGS.md"
source_sha256: "56690b74b51bfa3870e0d4559ecceb61f12a2152ec7ef9d29a17653819eeebd1"
mirror_body_sha256: "56690b74b51bfa3870e0d4559ecceb61f12a2152ec7ef9d29a17653819eeebd1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings

Reviewed source revision: `cb6e00d633772de538498384a8b14d32d25f7915`.
Review lane: repository/spec research only. The application was not launched for this task; no runtime defect is claimed as reproduced.

## Source-backed facts

| Fact | Evidence at reviewed revision | Consequence |
|---|---|---|
| The active app is native eframe/egui; there is no active WebView UI. | `docs/architecture/system-overview.md:7-16` | Runtime research must use `db-pro-native`, not archived frontend material. |
| DB Pro has a core component layer and view code must use it where a component exists. | `RULES.md:3-5,7-33` | No external UI framework is a prerequisite for this task; first reuse the project component contracts. |
| The shared tab handler treats arrow keys as selection movement, wraps the index, and handles activation separately. | `crates/ui/src/components/tabs/handler.rs:1-34` | Test reported “tab focus” as two possible contracts: widget focus traversal vs. tab selection/activation. This alone does not prove end-to-end behavior. |
| Existing handler tests cover wraparound, empty/out-of-range tracks, and keyboard-vs-click precedence. | `crates/ui/src/components/tabs/tests/handler_tests.rs:4-33` | Reuse this coverage; investigate cross-view focus ownership at runtime rather than duplicating pure handler tests. |
| Modal traversal and connection-dialog interaction already have a separate active plan; live keyboard interaction remains pending there. | `docs/plans/active/connection-dialog-accessibility/PLAN.md:27-41,66-82` | Coordinate/reuse; do not claim that the existing plan's modal code is runtime-proven. |
| Application-wide native focus/accessibility traversal remains open in component-layer work; broad visual/layout changes are active elsewhere. | `docs/plans/active/component-layer-structure/CHECKLIST.md:15,26-29`; `docs/plans/active/native-feature-ui-cleanup/PLAN.md:6-19` | Keep this effort centered on verified focus/runtime behavior and avoid parallel style-system or general redesign work. |

## Existing runtime artifact review

Inspected prior native captures before proposing any new runtime run:

- `docs/plans/active/component-interaction-polish/evidence/input-selected-native.jpg` and `alert-selected-native.jpg`: visible blue focus border/selected text in the Database input; alert description selection uses blue background and white text. `VERIFICATION.md:32` records these as native GUI observations and says clicking Dismiss removes the summary while validation errors remain. This proves those specific visible/click states, not Tab/Shift+Tab order or cross-view focus restoration. The associated exact source SHA is `cd58b354a43f55458759c298bc053c5a4dd36c0b` (`AGENT_EVIDENCE.md:11,16`).
- `docs/plans/active/component-interaction-polish/evidence/form-error-1280x800.png`: error/validation form layout at a native framebuffer; not keyboard traversal evidence.
- `docs/plans/active/native-feature-ui-cleanup/evidence/welcome-agent-light-1280x800.png`, dark counterpart, and `query-agent-dark-1280x800.png`: real rendered shell/Welcome/Query/Agent composition. No discernible focused control or traversal sequence is exposed in these static captures. The adjacent `welcome-agent-light-1280x800.log` records 2026-10-19 capture warnings, including a failed SQLite sample fixture and a missing PostgreSQL database; those runtime messages are not evidence of a focus defect. The inspected image/log pair does not identify an exact source SHA.

Artifact review therefore narrows the gap: focus styling is visibly represented in the focused input artifact, while application-wide keyboard traversal, focus ownership after switching tabs, and focus restoration remain unproven by the inspected materials. Do not recapture surfaces already covered; first look for exact-path interaction records in the associated plan verification files. Collect new evidence only for unresolved cases.
- `docs/plans/active/native-visual-redesign/VERIFICATION.md:134-143` records a prior native SQLite grid keyboard smoke: selecting `name`, then Right, Down/End, visibly moved the active-cell accent to `status` and `profile` on row 2. It proves the documented grid path at that historical run, so do not repeat it by default. The screenshot is referenced under a host temporary `/var/folders/...` path, and the record does not attach an exact source SHA; treat it as historical evidence, not proof of current-build behavior.

## Evidence coverage audit — 2026-10-20

| Contract | Existing proof | Assessment |
|---|---|---|
| Modal Tab / Shift+Tab between inputs | `connection-dialog-accessibility/VERIFICATION.md:31-41` records `components::dialog::modal_guard::tests::tab_advances_focus_between_fields_inside_modal` PASS (1/0/0) at implementation SHA `92746074431b60f2f96347c940293da10952b292`. The test injects forward Tab and reverse Shift+Tab into egui frames and asserts focus moves second→first inside the overlay. | Reuse as automated regression evidence for the modal component. It is not real-window keyboard evidence; same verification record lines 104-114 explicitly leaves live Tab/Shift+Tab pending because the Orca invocation was unavailable. Do not rerun this automated test merely to duplicate the record. |
| Native grid active-cell navigation | `native-visual-redesign/VERIFICATION.md:134-143` records a native SQLite smoke using Right then Down/End; the active cell visibly moves from `name` to `status`, then `profile` on row 2. | Historical flow already exercised; do not repeat by default. Screenshot is in a host-temporary path and the row has no source SHA, so it cannot establish behavior at the reviewed/current revision. |
| Focus / selection appearance | `component-interaction-polish/VERIFICATION.md:32` documents native selected-input and alert captures; its AGENT_EVIDENCE records implementation SHA `cd58b354a43f55458759c298bc053c5a4dd36c0b`. The input image visibly shows a blue focus border and selected text. | Reuse for those specific visual states; no traversal path is proven. |
| App-wide widget Tab order and focus after workspace/document tab switching | The surveyed `connection-dialog-accessibility`, `component-layer-structure`, and `native-feature-ui-cleanup` records explicitly leave native keyboard traversal pending. No exact-path runtime record surfaced in those plans. | Still unproven. Existing captures do not justify recapturing these views, but do not close keyboard interaction acceptance. |

The plan therefore does not require repeating the old grid smoke or the modal unit test. If follow-up is requested, restrict new runtime interaction to the uncovered path(s): actual application Tab/Shift+Tab outside the already-tested modal, and focus ownership after switching workspace/document tabs. Use the reported screen/path if it becomes identifiable; no broad all-screen re-run is justified by these records.

## Current verdict

- No application-wide P0/P1/P2 focus defect is confirmed by these artifacts.
- Existing captures prove selected/focused visual states for specific components, but static images do not prove a keyboard sequence.
- Native-feature captures show realistic shell/content composition; the visible database failure banner is fixture/runtime context, not a verified UI focus or layout root cause.
- Existing tab handler unit tests and screenshots do not establish application-wide keyboard traversal at runtime.
- No code, tests, styles, dependencies, or provider paths were changed.


## Recommended decision

Before launching the app or recapturing UI, inspect related plan verification records for exact-path keyboard events and source SHA. Run additional interaction only where existing evidence is absent or does not prove the required focus invariant. For each actual mismatch, trace component/caller ownership and distinguish interaction ownership, widget-ID collision/reuse, state transition, visibility, and focus styling; prefer the narrowest owning fix over a generic global focus manager.

## Remaining questions

- Which concrete screen/control sequence first exhibits the reported focus behavior?
- Does the issue reproduce in both light and dark themes and with mouse vs. keyboard as the last input method?
- Does switching workspace/document tabs retain focus on a hidden view, or does focus move to a visible control?
- Which host/OS/input path is available for interactive keyboard verification, and what effective logical sizes can it render?

## Provider impact

PostgreSQL: N/A. SQLite: N/A. This plan is presentation/interaction-only and does not alter database behavior.
