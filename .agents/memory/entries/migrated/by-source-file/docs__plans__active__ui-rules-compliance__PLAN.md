---
schema_version: 1
id: "MEM-SRC-8c319c9208277d6d"
title: "Source mirror: docs/plans/active/ui-rules-compliance/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-rules-compliance/PLAN.md"
source_sha256: "8c319c9208277d6d1bee5802b5b039265b077c74e0b1c50de9e5f9997907ba97"
mirror_body_sha256: "8c319c9208277d6d1bee5802b5b039265b077c74e0b1c50de9e5f9997907ba97"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# PLAN — ui-rules-compliance

## Goal

Make `crates/ui` conform to `RULES.md` (component-first UI rules) at HEAD
`445bb165ed0d2c507c226574658185a21df2f5cc`, and make the repo's own declared gate
(`RULES.md` §6) pass instead of the narrower `scripts/check-ui-architecture.sh`.

## Non-goals

- No visual redesign. Every replacement MUST render the same surface, same strings,
  same widget ids, same order. Any intentional visual delta must be listed in
  VERIFICATION.md with a screenshot.
- No change to `components/` public API beyond adding a missing primitive when no
  existing component fits a pattern.
- No `RULES.md` text change, no weakening of the rule.
- §3 (hard-coded colours/metrics) is a separate wave — see CHECKLIST phase W2.

## Scope (frozen)

`docs/plans/active/ui-rules-compliance/FINDINGS.md` — 88 non-test §1 sites:

| bucket | count | action |
|---|---:|---|
| `frame (styled)` | 61 | replace with `components::legacy::*_frame` helper / `components::card` / `components::badge` / `components::hover_card` |
| raw widget (`ui.small_button`, `egui::Button`, `ui.radio_value`, `egui::Area`, `CollapsingHeader`, `ui.checkbox`, `egui::ScrollArea`) | 23 | replace with the matching core component |
| `frame (layout-only)` | 4 | keep — §2 allows `ui.allocate_*`/layout-only frames |

Plus: `RULES.md` §6 gate command must return empty output, which also requires the
test-file and `component_gallery_rendering.rs` usages to carry an inline
`// ui-rules: <reason>` (same line) instead of relying on the script's path exemptions.

## Fix policy (binding for every agent)

1. If a core component exists for the pattern, it MUST be used. Reuse order:
   `components::legacy::{panel_frame, sidebar_frame, activity_bar_frame, toolbar_frame, tab_frame,
   card_frame, agent_message_frame, editor_frame, grid_frame}` →
   `components::card` / `components::badge` / `components::hover_card` / `components::button` /
   `components::selection` / `components::collapsible` / `components::overlay` / `components::tabs`.
2. If no existing component fits, add the smallest missing primitive under
   `crates/ui/src/components/` (theme-aware, light+dark, all five states) and use it.
3. A `// ui-rules: <reason>` comment may only be added when the site is genuinely one of
   §5's exemptions (inside `components/`, `dev_tools/`, `components::scroll_area` wrapper,
   editor/canvas paint, dev-only gallery reference baseline, test harness). Adding a comment
   where a component replacement exists is a REJECT and will be reverted.
4. Behaviour-preserving: same label text, same `Id`, same call order, same callbacks.
5. Minimal diff. Do not reformat unrelated lines. Do not touch files outside your slice.

## Evidence

Source: `FINDINGS.md` inventory (file:line). Automated: `RULES.md` §6 command empty,
`scripts/check-ui-architecture.sh` PASS, `cargo fmt/check/clippy/test`. Runtime: `capture`
feature screenshots light+dark at 1280×800 / 1440×900 / 1920×1080 for the affected surfaces
(recorded in VERIFICATION.md).
