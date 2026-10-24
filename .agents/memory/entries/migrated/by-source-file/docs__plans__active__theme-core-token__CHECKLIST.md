---
schema_version: 1
id: "MEM-SRC-cd63289f20e261d2"
title: "Source mirror: docs/plans/active/theme-core-token/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/theme-core-token/CHECKLIST.md"
source_sha256: "cd63289f20e261d25d591a824c6e558f03779b333f0e9a5436976de53384606e"
mirror_body_sha256: "cd63289f20e261d25d591a824c6e558f03779b333f0e9a5436976de53384606e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

## Slice 1 — numeric consolidation (previous)

- [x] Inspect current theme, shared tokens, and component token duplication.
- [x] Preserve existing public `DbProTheme` field API.
- [x] Route theme style mapping through shared tokens.
- [x] Route button config through shared button tokens.
- [x] Resolve clean-code review findings in the core mapping.
- [x] Run formatting and targeted compilation.

## Slice 2 — three-layer core contract (this slice)

- [x] Separate primitive palettes from semantic theme construction
  (`tokens/primitive.rs` private palettes; `DbProTheme::light/dark` rebuilt from
  `SemanticTokens`).
- [x] Every semantic role has light + dark treatment (unit test
  `every_role_group_has_light_and_dark_treatment`).
- [x] Status roles provide solid / subtle / fill / border / foreground and are
  consumed by alert rendering through `theme.semantic.status.*`.
- [x] Compatibility aliases documented with one authoritative role
  (`surface_2 → background.subtle`, `text_muted → foreground.muted`).
- [x] `EGUI_*` / `RADIUS_EGUI_*` adapter constants private to `theme.rs`.
- [x] Button single source of truth: height, padding, font size, glyph size,
  radius live in the Button contract; hit-box sizes separated from glyph sizes.
- [x] Component contracts for Button, Input, Dialog/Overlay, Table/Grid,
  Feedback/Status with supported states and explicit precedence.
- [x] State precedence `disabled → loading → active → focus → hover → default`
  declared (`component::STATE_PRECEDENCE`) and pinned by tests.
- [x] Fix disabled+loading button rendering the spinner
  (`button_contract::shows_loading`).
- [x] Fix disabled input wearing focus/error borders (`resolve_chrome`).
- [x] Focus ring painted from `border.focus` at `STROKE_THICK` (same values).
- [x] Existing rendered values unchanged — pinned by theme/button/input tests
  (928 workspace tests pass).
- [x] Gates: `cargo fmt --all -- --check`, `cargo check -p db-pro-ui`
  (+ `cargo check --workspace`), clean-code scan (0 failures),
  `git diff --check`.
- [ ] UI runtime capture at 1280×800 / 1440×900 / 1920×1080 — **NOT VERIFIED**
  (no UI runtime session in this slice).
- [x] Record remaining component-local token migration separately
  (see FINDINGS "Deferred").
