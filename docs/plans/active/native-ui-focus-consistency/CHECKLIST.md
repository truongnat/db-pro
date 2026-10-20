# Checklist — Native UI Focus and Consistency Audit

## PLANNING / research

- [x] Inventory and inspect representative existing native artifacts before requesting any new UI run: component-interaction selected-input/alert and form-error captures; native-feature-cleanup Welcome/Agent and Query/Agent captures plus logs; native-visual-redesign's historical native grid-keyboard record.
- [x] Reconcile findings/scope from `connection-dialog-accessibility`, `component-layer-structure`, `component-interaction-polish`, and `native-feature-ui-cleanup`; no changes duplicate their accepted fixes.
- [x] Match contracts to existing evidence: modal Tab/Shift+Tab egui test PASS at `92746074431b60f2f96347c940293da10952b292` (not live-window proof); old grid Right/Down/End smoke is recorded but lacks source SHA; focused input visuals trace to `cd58b354a43f55458759c298bc053c5a4dd36c0b`; no revision-traceable workspace/document tab-switch focus record found in the surveyed plans. New interaction is limited to these gaps.
- [ ] Identify the affected surface(s) from the reported incident and prepare deterministic startup state/data only if existing evidence does not cover the path.
- [ ] Record current focus owner/outcome for any still-unverified Tab, Shift+Tab, arrow navigation, Enter/Space, tab switch, overlay open/close, or editor/form transition. Reuse prior recorded interaction evidence when it covers the exact path.
- [ ] For remaining visual-focus gaps, use existing captures first; only add Light/Dark native capture where focus appearance cannot be assessed from the available evidence. Record source SHA, effective viewport, and artifact paths.
- [ ] Trace each evidenced mismatch to its owner component/caller and check existing regression coverage. Mark non-reproduced or screenshot-only concerns unconfirmed; do not code from suspicion.
- [ ] Update `FINDINGS.md` with one row per root cause, severity, proof, affected shared callers, existing coverage, recommended smallest fix, and deferred items.

## IMPLEMENTING — only after a confirmed root cause

- [ ] Fix the narrowest owning component/caller using existing egui/core-component and theme patterns.
- [ ] Add a deterministic regression test for each important consumer-visible focus invariant; tests are not substitutes for native interaction.
- [ ] Preserve current user actions, shortcut meaning, public API, and data safety behavior.
- [ ] Run the focused UI tests and applicable format/check/architecture/release gates; record exact results and failures.

## RUNTIME_VERIFY

- [ ] Reuse an existing revision-traceable native before record for any confirmed defect; capture/replay an original interaction only if no adequate before evidence exists, then verify the corrected path in the rebuilt `db-pro-native`.
- [ ] Verify focus order, visible focus indicator, no hidden-control focus leak, and modal/overlay focus behavior for affected paths.
- [ ] Check Light and Dark on affected states; capture target 1280×800, 1440×900, 1920×1080 when available, and record effective dimensions/host caps.
- [ ] Record failures, skipped evidence, exact commands, artifacts, unresolved P0/P1/P2, and provider applicability in `VERIFICATION.md`.
- [ ] Leave the plan in `RUNTIME_VERIFY` until native interaction evidence and all applicable acceptance criteria are met; update `STATUS.md` only when lifecycle evidence supports the state.
