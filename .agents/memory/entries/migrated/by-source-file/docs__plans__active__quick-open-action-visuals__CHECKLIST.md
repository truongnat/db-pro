---
schema_version: 1
id: "MEM-SRC-affe2ee9f6366a6e"
title: "Source mirror: docs/plans/active/quick-open-action-visuals/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/quick-open-action-visuals/CHECKLIST.md"
source_sha256: "affe2ee9f6366a6e6a9058ef6fbd1d00195a43f2c19e032bc56f9cb4239b006a"
mirror_body_sha256: "affe2ee9f6366a6e6a9058ef6fbd1d00195a43f2c19e032bc56f9cb4239b006a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

- [x] Reproduce selected Quick Open text resolving to primary foreground instead of selected foreground.
- [x] Apply explicit selection foreground to selected scope labels.
- [x] Remove Quick Open selected-row stroke and scope-filter hover/active strokes.
- [x] Remove Button strokes from Outline, disabled, and loading states; retain keyboard focus ring.
- [x] Add regression tests for light/dark selected foreground and all Button variants/states.
- [x] Capture Quick Open normal/empty states and Button Gallery loading/disabled states.
- [x] Run Rust workspace quality gates and native release build.
- [x] Review native captures at 1280×800 and available 1440/1920 widths.
- [ ] Verify full 1440×900 and 1920×1080 logical heights on a display that supports them.

## Owner-reported regressions — 2026-10-19

- [x] Disable duplicate idle shell separators while retaining semantic subtle boundaries.
- [x] Synchronize toolbar/Settings/Gallery theme selection; explicitly select egui theme and refresh root UI style.
- [x] Pass pointer-driven toolbar/Settings theme switching, repeated-frame, and save/restore regression.
- [x] Reproduce then fix SearchInput consuming a tall container's remaining height.
- [x] Keep empty Quick Open compact and keyboard help outside the scrolling body; regression fails before/passes after.
- [x] Build native capture and release binaries; final app/UI suite: 1299 passed, 0 failed, 0 ignored.
- [x] Capture and inspect 36 final light/dark images: shell, Agent, Quick Open, typed no-match, pending test, validation error at all three requested widths.
- [x] Record workspace-test, Clippy, capture shutdown, and exact-height limitations without calling failed gates passing.
- [ ] Obtain full-height 1440×900 / 1920×1080 captures (current framebuffers cap at 838 logical pixels).
- [ ] Independent review and real native pointer resize/hover verification.

## Border follow-up

- [x] Remove explicit outer frame strokes in addition to egui idle separator lines.
- [x] Rendered-shell regression covers both themes, repeated frames, Agent, output open/closed.
- [x] Capture 12 native shell/Agent images across requested widths; all exit 0; inspect actual images.
- [ ] Full requested heights and true native pointer resize/hover remain subject to the previously recorded limitations.
