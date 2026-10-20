# Native UI focus and consistency audit — specification

State: PLANNING. This file defines the investigation and implementation contract; it does not assert that a runtime defect has been reproduced.

## User need

The shipped desktop UI feels inconsistent when rendered in the real application. Keyboard focus and tab movement are specifically reported as unpredictable. Determine where the behavior diverges, prove it against the running native app, then make only evidence-backed fixes using the existing egui component/theme system.

## Requirements

- R1 — Inventory and inspect existing native screenshots, capture logs/manifests, test results, and active plan evidence before scheduling new UI runs. Prefer evidence already collected; request additional interaction only for questions existing artifacts cannot answer.
- R2 — Establish what evidence shows for keyboard navigation, distinguishing workspace/document tab selection from keyboard focus traversal between controls. Record evidence source/revision and its limits.
- R3 — Audit a representative path through shell navigation, document tabs, form fields, overlays/dialogs, and data-grid/editor surfaces; link existing artifacts first, then record any remaining exact steps, observed focus owner, expected owner, theme, viewport, and source revision.
- R4 — Identify root causes and affected callers before changing code. Reuse existing core components, tokens, and egui focus behavior; no parallel component framework.
- R5 — Fix confirmed focus-order, focus-stealing, focus-retention, or focus-restoration defects with regression coverage at the responsible component/state boundary. Preserve mouse actions and existing keyboard shortcuts.
- R6 — Verify the corrected path with existing runtime evidence where it fully covers the behavior; use the running native binary only for missing interaction evidence. Record screenshot/capture limitations separately from interactive evidence.
- R7 — Keep visual cleanup consistent with current `DbProTheme`, component-first UI rules, and existing active visual-cleanup work; do not expand this into an indiscriminate redesign.

## Context and current evidence

- The product UI is Rust `eframe`/`egui` in `crates/ui` and `crates/native-app`; there is no active WebView UI. Source: `docs/architecture/system-overview.md` lines 7–16, baseline `cb6e00d633772de538498384a8b14d32d25f7915`.
- The project already has a core component layer and requires views to use it: `RULES.md` lines 3–5, 7–33, baseline `cb6e00d633772de538498384a8b14d32d25f7915`. Thus “no external UI framework” is not evidence that a new framework is needed.
- The tabs handler has explicit arrow-key selection and activation rules (`crates/ui/src/components/tabs/handler.rs` lines 1–34); tests cover wrapping, empty/out-of-range tracks, and keyboard-vs-click precedence (`crates/ui/src/components/tabs/tests/handler_tests.rs` lines 4–33), same baseline. These source/unit facts do not prove application-wide keyboard traversal is correct at runtime.
- The separate `connection-dialog-accessibility` plan records the modal Tab/Shift+Tab egui-frame test as PASS at implementation SHA `92746074431b60f2f96347c940293da10952b292`, but its `VERIFICATION.md` still lists real-window keyboard interaction as pending. Reuse the test result; do not repeat it solely for this audit.
- `native-feature-ui-cleanup` is implementing cross-surface visual/layout fixes; `component-layer-structure` still lists native keyboard/focus/accessibility traversal as pending. This spec must reuse their findings and avoid re-auditing or duplicating their visual/code scope.

## In scope

- Observe real keyboard focus and tab-selection behavior on the native app's highest-risk shared paths.
- Trace each reproducible issue to its component, caller, focus ID/ownership, state transition, and test surface.
- Implement only confirmed defects in the smallest owning component/caller, with a focused regression test and runtime proof.
- Capture visual focus/selected states when the issue requires visual evidence.

## Out of scope

- Replacing egui or adding an external UI/component framework.
- Rebuilding the component library, changing the product palette, or broad restyling already covered by `native-feature-ui-cleanup` / `core-component-style-system`.
- Changing SQL, provider, runtime service, persisted-state, or database behavior.
- Treating screenshots, source inspection, or headless widget tests as proof of real keyboard traversal.
- Fixing unobserved visual inconsistencies merely because they look suspicious in source.

## Definition of done

- [ ] An evidence matrix separates Tab focus, Shift+Tab, arrow-key tab selection, Enter/Space activation, modal focus containment/restoration, and focus after workspace/document tab switching. Existing runtime records count only when they document the exact path, behavior, and source revision; request new interaction only for gaps.
- [ ] Each claimed defect has reproducible steps, expected/actual behavior, exact source revision, and identified root cause; non-reproduced reports remain explicitly unconfirmed.
- [ ] Confirmed defects are fixed at the narrowest owning component/caller; regression coverage proves the user-visible invariant without pinning implementation details.
- [ ] Each in-scope flow has valid native keyboard evidence from existing revision-traceable records or a targeted runtime check for the remaining gap; screenshots alone never prove keyboard traversal.
- [ ] Light/dark focus indicators remain visible and use existing theme tokens; no raw widget bypass or duplicated styling convention is added.
- [ ] Existing overlapping plans are linked and their scope is not duplicated; PostgreSQL and SQLite are independently marked N/A because this is UI-only.
- [ ] `VERIFICATION.md` records only checks actually executed, evidence type, host viewport limitations, unresolved findings, and lifecycle state. Stay active until runtime evidence is complete.
