---
schema_version: 1
id: "MEM-PLAN-native-ui-focus-consistency-8ac15211cb1d"
title: "Plan archive: Native Ui Focus Consistency"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-ui-focus-consistency]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "8ac15211cb1d24cc047c566eee21287dcad4a8c490d355ad2be3c446bf372f22"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-ui-focus-consistency/PLAN.md"
  - "docs/plans/active/native-ui-focus-consistency/FINDINGS.md"
  - "docs/plans/active/native-ui-focus-consistency/VERIFICATION.md"
  - "docs/plans/active/native-ui-focus-consistency/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-ui-focus-consistency/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native UI Focus and Consistency Audit

State: PLANNING. Research/spec only in this change; implementation and runtime behavior are not yet verified.
Baseline source revision reviewed: `cb6e00d633772de538498384a8b14d32d25f7915`.

## Goal

Diagnose the reported unpredictable keyboard focus/tab behavior in the actual shipped native app and repair only reproducible defects. Improve confidence in cross-surface coherence by verifying focus/selection states in runtime, rather than prescribing a broad visual rewrite.

## Design and interaction contract

- Product: native database IDE for desktop keyboard/mouse workflows; Rust `eframe`/`egui` only.
- Treat “tab” as two separate concepts during research: (1) keyboard focus traversal with Tab/Shift+Tab, and (2) selection/navigation among workspace or document tabs, generally with arrow keys/activation.
- Focus has a current owner. Opening an overlay/modal moves focus inside according to its existing contract; traversal remains within modal where required; closing returns focus to its trigger when supported. Switching a workspace/document tab must not leave stale focus on a hidden control or steal focus from active text input unexpectedly.
- Existing core components, `DbProTheme`, and UI rules are authoritative. No new UI framework, global focus manager, or component abstraction unless findings prove the existing ownership model cannot meet a specific acceptance criterion.

## Evidence and scope boundaries

- Current behavior is PLANNING evidence only. `SPEC.md` records source references and known overlapping work.
- Reuse current results from `connection-dialog-accessibility`, `component-layer-structure`, `component-interaction-polish`, and `native-feature-ui-cleanup`; do not repeat their already-passing implementation work.
- PostgreSQL: N/A. SQLite: N/A. No database behavior changes.
- Visual scope is limited to focus ring/selected-state legibility on the audited paths. Broad spacing, chrome, palette, and component styling changes belong to existing plans.

## Work phases

1. **Review artifacts first:** inventory screenshots, capture logs/manifests, test reports, and previous audit findings; inspect focus-relevant examples before scheduling any new native run. Reuse sufficient, revision-traceable evidence.
2. **Close evidence gaps only:** map what each existing artifact actually proves. Run interactive native keyboard steps only when no existing artifact/test record proves the required behavior or when the reported path is still ambiguous.
3. **Trace and decide:** map each evidenced mismatch through component → caller → focus identity/state → visible outcome. Check existing plans/coverage and distinguish screenshot appearance from interactive traversal. Reject unconfirmed/speculative defects.
4. **Minimal correction:** change the smallest responsible component/caller, add one high-value regression per affected invariant, and preserve current public APIs/actions unless a proven defect requires otherwise.
5. **Verify and close:** reuse valid before/after runtime evidence where it covers the exact path; collect only missing post-fix evidence in the rebuilt binary. Keep `RUNTIME_VERIFY` if required evidence remains outstanding.

## Acceptance criteria

- Confirmed navigation defects are reproducible before a fix and pass the same native scenario afterward.
- Keyboard focus order is predictable through representative shell/navigation, editor/form, and overlay/modal flows; hidden/inactive controls do not retain misleading focus.
- Workspace/document tab selection and ordinary Tab focus are documented and tested as distinct contracts.
- Modal/overlay focus containment and return behavior remain correct on the audited paths; existing dialog plan evidence is reused rather than claimed anew.
- Focus/selected visuals remain perceptible in Light and Dark with existing tokens; no DB Pro UI rule violations are introduced.
- Tests and runtime evidence are recorded separately, and any inaccessible host interaction/evidence remains NOT VERIFIED rather than inferred.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| “Tab is broken” refers to multiple different keyboard behaviors | Record exact key, focused control, selected tab, and expected transition for each report. |
| Headless egui tests give false confidence about native traversal | Require an actual native keyboard-only run for the affected flow; tests are regression support, not runtime proof. |
| Overlap with active visual/component initiatives creates conflicting edits | Reconcile their plans first; route purely visual findings to their owner plan or explicitly defer them. |
| Current host limits viewport sizes or capture process exits | Record requested vs effective viewport and process status; do not claim exact-size or interaction evidence from a static capture. |

## Plan files

- `SPEC.md` — requirements, current context, scope, and DoD.
- `CHECKLIST.md` — ordered research/implementation/verification steps.
- `FINDINGS.md` — evidence-backed findings and decisions.
- `VERIFICATION.md` — execution record; currently documents-only.

## Tổng kết bằng tiếng Việt

Đây là plan điều tra và xử lý có bằng chứng cho lỗi focus/tab trên UI egui đang chạy thật. Tách rõ Tab/Shift+Tab khỏi việc chọn workspace/document tab; tái sử dụng component/theme và kết quả các plan đang active, không thêm framework hoặc tổng redesign. Chưa kết luận lỗi runtime đã tái hiện hay được sửa.


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Verification

## Current pass — specification only

- Source baseline recorded: `cb6e00d633772de538498384a8b14d32d25f7915`.
- Read lifecycle/status, native UI architecture, UI rules, focus/tab component source and tests, and overlapping active plans.
- Reviewed existing native artifacts and records: `component-interaction-polish/evidence/input-selected-native.jpg`, `alert-selected-native.jpg`, `form-error-1280x800.png`; `native-feature-ui-cleanup/evidence/welcome-agent-light-1280x800.png`, its dark counterpart, `query-agent-dark-1280x800.png`, and adjacent capture log; `native-visual-redesign/VERIFICATION.md:134-143` historical native SQLite grid-keyboard smoke.
- Artifact conclusion: component-interaction captures show focused/selected styling and a documented native click state; the older native-visual-redesign record documents Right → Down/End active-cell navigation in the grid, but its screenshot path is host-temporary and its exact source SHA is absent. Neither provides Tab/Shift+Tab traversal or focus after switching workspace/document tabs. Welcome capture log includes fixture/database errors, not evidence of a focus fault. The component-interaction artifacts trace to SHA `cd58b354a43f55458759c298bc053c5a4dd36c0b`; the inspected native-feature screenshot/log pair lacks an exact source SHA.
- Documentation deliverables: `SPEC.md`, `PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`, and `AGENT_EVIDENCE.md`.
- No source, tests, dependencies, runtime UI, or provider behavior changed. Existing screenshots were inspected; no new capture was run.
- Automated tests/builds: not run (documentation/evidence-review pass).
- Live keyboard traversal was not run in this pass. The historical grid smoke is recorded in `FINDINGS.md` and should not be repeated by default; existing artifacts do not prove end-to-end Tab/Shift+Tab or post-tab-switch focus. No defect is claimed as reproduced.
- PostgreSQL: N/A. SQLite: N/A.
- Current state: PLANNING; do not mark COMPLETED. Runtime evidence is a required future gate.

## Existing verification records audited — 2026-10-20

- `connection-dialog-accessibility/VERIFICATION.md:31-41`: modal Tab and Shift+Tab test recorded PASS (1 passed / 0 failed / 0 ignored) at implementation SHA `92746074431b60f2f96347c940293da10952b292`. It injects keys into egui frames; it is automated component evidence, not a native-window interaction.
- The same record at `:104-114` and `:109-118` explicitly leaves live keyboard interaction pending. The Orca command was unavailable in that earlier environment; this is recorded history, not a new failure in this pass.
- `native-visual-redesign/VERIFICATION.md:134-143`: previous native SQLite grid key smoke is recorded as Right then Down/End, moving active-cell emphasis from `name` to `status` then `profile` on row 2. The image path is a host temporary path and the record does not attach a source SHA. This path is not scheduled for repeat by default.
- `component-interaction-polish/AGENT_EVIDENCE.md:11,16` and `VERIFICATION.md:32`: selected-input/alert native visual evidence is tied to implementation SHA `cd58b354a43f55458759c298bc053c5a4dd36c0b`; proves visible focus/selection states and a click dismissal, not keyboard traversal.
- `component-layer-structure/VERIFICATION.md:494-496` records native keyboard/focus traversal as NOT RUN. `native-feature-ui-cleanup` captures provide visual surfaces but no keyboard sequence.
- Evidence verdict: automated modal Tab/Shift+Tab is covered; visual focus appearance is covered for specific controls; historical grid navigation is recorded but revision-unqualified. Actual native-window Tab traversal outside the modal and focus after switching workspace/document tabs remain unproven in the surveyed plans.
- No commands, tests, builds, new captures, app launch, or key injection were run in this pass. Plan remains PLANNING; no implementation or defect attribution.

## Required future evidence

Record exact commands and outcomes, binary/source revision, actual keyboard interaction steps, focus owner before/after each key, Light/Dark state, requested and effective viewport, artifact locations, and any failures or skipped cases. Static screenshots do not prove Tab/Shift+Tab traversal.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Native UI Focus and Consistency Audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · research/specification lane |
| Issue(s) | Owner request: investigate and specify native UI cleanup/focus behavior |
| Task state | Ready |
| Baseline SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Branch / PR | `main` (owner-mandated workspace; documentation only) |
| Scope interpretation | Write a bounded investigation and acceptance spec for real native focus/tab behavior; no implementation in this task. |
| Out of scope | Broad UI restyle, external framework/dependency, database/provider changes, unverified defect claims. |

## 2. Progress checkpoint

- Current reviewed source revision: `cb6e00d633772de538498384a8b14d32d25f7915`.
- Completed acceptance rows: [x] mapped native UI architecture and existing focus/tab tests/plans; [x] reviewed representative captures; [x] audited modal/grid runtime and automated records with their provenance; [x] wrote/updated SPEC/PLAN/CHECKLIST/FINDINGS/VERIFICATION; [x] registered PLANNING state in STATUS.md.
- Remaining acceptance rows: [ ] native-window Tab/Shift+Tab outside the modal and focus ownership after workspace/document tab switching, if still required after the affected path is identified; any resulting implementation/runtime verification.
- Findings / risks: the modal egui Tab/Shift+Tab test is documented PASS at `92746074431b60f2f96347c940293da10952b292`; live modal key interaction is explicitly pending. Grid key smoke is historical but lacks a source SHA. No application-wide defect is confirmed.
- Tests already run: none in this audit; documentation and evidence-record review only. Existing documented test results were not rerun. Native UI not launched.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Commit list | none |
| File / surface inventory | `docs/plans/active/native-ui-focus-consistency/{SPEC,PLAN,CHECKLIST,FINDINGS,VERIFICATION,AGENT_EVIDENCE}.md`; added requirements/context/DoD, evidence-first research steps, boundaries, source/artifact findings, and verification status. `docs/plans/STATUS.md`; registered feature as PLANNING. |
| Acceptance mapping | Owner's request → SPEC/PLAN; action path → CHECKLIST; source/artifact evidence and unknowns → FINDINGS; inspected existing artifacts and not-run gates → VERIFICATION. |
| Commands and counts | `git rev-parse HEAD` → `cb6e00d633772de538498384a8b14d32d25f7915`; no test/build/runtime command run. |
| CI run IDs / status | not run |
| Known limitations | No current native-window interaction. Existing modal test is automated only; historical grid screenshot is host-temporary and lacks source SHA; workspace/document tab-switch focus evidence was not found in surveyed plans. No implementation. |
| Migrations / config implications | none |
| Out-of-scope changes | no source/test/style/dependency changes |

## 4. Review outcome

n/a — no independent review requested or performed.

## 5. Research / audit handoff

- Source date: 2026-10-20.
- Source URLs / references: `docs/architecture/system-overview.md:7-16`; `RULES.md:3-5,7-33`; `crates/ui/src/components/tabs/handler.rs:1-34`; `crates/ui/src/components/tabs/tests/handler_tests.rs:4-33` (reviewed source at baseline SHA above). Plan evidence: `connection-dialog-accessibility/VERIFICATION.md:31-41,104-118` and `AGENT_EVIDENCE.md:17-20,29-34`; `component-layer-structure/VERIFICATION.md:494-496`; `component-interaction-polish/AGENT_EVIDENCE.md:11,16` and `VERIFICATION.md:32`; `native-visual-redesign/VERIFICATION.md:134-143`; `native-feature-ui-cleanup/VERIFICATION.md` / captures.
- Factual findings: modal Tab/Shift+Tab has a passing egui-frame test at the implementation SHA above, but its active plan explicitly leaves real-window keyboard interaction pending. The historic native grid smoke covers cell navigation only and has incomplete provenance. Focus appearance captures are tied to a different recorded SHA; shell/agent screenshots do not show a key traversal.
- Inference: “tab focus” may refer to focus order or tab selection; no surveyed record proves focus ownership after switching workspace/document tabs.
- Decision / recommendation: preserve the passing modal/grid proof; do not repeat those scenarios by default. New interaction is only justified for the unproven actual-app Tab path outside the modal and workspace/document tab-switch focus, once those paths are identified.
- Unresolved questions: which exact surface/path the user calls “tab focus”; whether the observed issue overlaps those two gaps; native host/source revision for any needed future run.
- Downstream tasks activated: resolve only uncovered acceptance gaps in `CHECKLIST.md`; no implementation task started.

## 6. Tổng kết bằng tiếng Việt

Đã quét các capture focus/form, ảnh Welcome/Query/Agent và hồ sơ keyboard-grid có sẵn. Bằng chứng cũ xác nhận một số trạng thái focus/selected và một luồng điều hướng grid, nhưng chưa chứng minh Tab/Shift+Tab hay focus sau khi đổi workspace tab; không chạy lại luồng đã có evidence phù hợp. Spec đã đổi sang kiểm tra evidence trước và chỉ xác minh runtime các khoảng trống còn lại; chưa sửa code.

