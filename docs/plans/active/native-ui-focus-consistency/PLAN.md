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
