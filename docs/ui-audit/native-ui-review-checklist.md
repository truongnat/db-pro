# DB Pro native UI review checklist

Use for changes to the shipped Rust/egui UI. This checklist separates deterministic correctness from visual comparison and human judgment. A screenshot diff reports change; it does not decide whether the change is good.

## Before review

- [ ] Name the surface, fixture/data, theme, viewport, and states affected.
- [ ] Classify each finding as a **UI defect**, **UX defect**, **visual preference**, or **redesign proposal**.
- [ ] For a bug fix, keep the patch within the defect's surface and behavior. Do not change the visual language or redesign adjacent UI without an explicit request.
- [ ] State the expected behavior and invariant before choosing the test.
- [ ] Use `DbProTheme`/semantic tokens as the source of truth. Do not impose a generic web spacing grid or arbitrary radius/size limits on egui.

## Gate 1 — Correctness

- [ ] Fixture data and UI state are the ones the scenario is meant to show.
- [ ] Required labels, values, actions, and status are present; state changes match the action taken.
- [ ] Empty, loading, and error behavior is appropriate where the changed surface supports those states.

## Gate 2 — Layout integrity

- [ ] Assert element/content bounds against the actual container and clip bounds at representative widths.
- [ ] No required column, header, action, or hit area is silently cut off or drawn outside its container.
- [ ] Truncation is intentional; the full value remains available through an appropriate tooltip or detail view.
- [ ] Define a constrained-width case for each multi-panel surface. Record which panels may shrink/collapse and the order in which optional content is sacrificed.
- [ ] Under constrained width, preserve primary actions and required identifiers/headers; never clip an action's hit area or silently remove it.
- [ ] Treat scrolling/clipping as intentional only when it belongs to the relevant scroll/container behavior.

## Gate 3 — Visual quality

- [ ] Capture the affected runtime surface at **1280×800**, **1440×900**, and **1920×1080**, plus any surface-specific constrained-width case.
- [ ] Capture relevant light/dark themes and normal/loading/error/empty states; record combinations that do not apply or could not be run.
- [ ] Review hierarchy, alignment, spacing, information density, contrast, and visual consistency against the product context and existing theme.
- [ ] Check typography: configured/resolved font family and weight where observable, text clipping, baseline alignment, glyph fallback, and readability of secondary/disabled labels.
- [ ] Use representative glyph fixtures (including supported non-ASCII text); inspect emoji or other fallback glyphs when the surface is expected to show them.
- [ ] Check scale factors **1.0, 1.25, 1.5, and 2.0** when the renderer/host can set them. If a factor cannot be exercised, record it as not run; do not infer its result from a different scale.
- [ ] Treat screenshot diffs as alerts for review. Approve a new baseline only after reviewing the changed image.

### AI visual findings

Every reported issue must include:

- [ ] Severity: `critical`, `major`, or `minor`.
- [ ] Surface and specific component/region.
- [ ] Screenshot filename, viewport/theme/state, and a region or coordinates identifying the evidence.
- [ ] Observable problem, expected result, and a specific recommendation.
- [ ] Confidence, clearly treated as an estimate.

Do not accept uncited judgments such as “spacing looks off.” The AI score may track trends, but it is not a pass/fail gate and cannot replace deterministic checks or human review.

## Gate 4 — Interaction quality

- [ ] Exercise relevant mouse actions, keyboard navigation, focus, scrolling, hover/tooltips, and visible feedback.
- [ ] Check accessible name/role/state for custom or icon-only controls where applicable.
- [ ] For important workflows, record the user goal, expected result, actual result, and any dead end or unexplained navigation.

## Review completion

- [ ] Correctness invariants pass.
- [ ] Layout/bounds and required constrained-width checks pass.
- [ ] Runtime screenshots were reviewed; any baseline update was approved by a human.
- [ ] Relevant interaction checks pass.
- [ ] **Zero unresolved critical issues and zero unresolved major issues.** Minor issues are listed with an owner or an explicit disposition.
- [ ] Report skipped checks and unsupported host/scale combinations as not run; never imply they passed.

## DB Pro references

- Theme tokens: `crates/ui/src/theme.rs` and `crates/ui/src/tokens/`.
- Native capture harness and deterministic fixtures: `crates/native-app/src/capture.rs`, `crates/ui/src/workspace_actions.rs`.
- Current geometry-test pattern: `crates/ui/src/app_tests.rs`.
- Existing viewport/state visual acceptance gate: `docs/10-egui-native-migration-plan.md`.
- Automation capabilities and current egui testing-library compatibility notes: `docs/ui-audit/automated-native-ui-checks-research.md`.
