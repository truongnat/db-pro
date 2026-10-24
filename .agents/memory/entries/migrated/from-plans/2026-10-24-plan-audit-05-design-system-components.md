---
schema_version: 1
id: "MEM-AUDIT-05-design-system-compone-2bf4eb243b8e"
title: "Plan archive: Audit 05 Design System Components"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, audit-05-design-system-components]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "2bf4eb243b8e65e42753ddd4256ce0090f5c33c22c5e7638b72661f33287e2cb"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/_audit-2026-10-13/05-design-system-components.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ audit slice `docs/plans/_audit-2026-10-13/05-design-system-components.md`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

# Audit — Design System & Components (slice 05)

Date: 2026-10-13. HEAD: `d73148cd`. Method: plan docs vs code at HEAD; read-only audit.

## 1. Slice scope

14 plans: theme-core-token, native-ui-component-core, core-component-style-system,
native-core-ui-modernization, native-shadcn-ui-system, component-layer-structure,
component-interaction-polish, component-gallery-redesign, gallery-layout-collisions,
gallery-form-controls, disclosure-components-polish, ui-component-language,
ui-core-audit, quick-open-action-visuals.

## 2. Per-plan verdicts

| Plan | Claimed state | Supported state | Verdict | Key evidence |
|---|---|---|---|---|
| theme-core-token | IMPLEMENTING; 3-layer contract done; runtime capture unchecked | Code supports all claims; only runtime capture missing | SUPPORTS (evidence honestly pending) | `tokens/{primitive,semantic,component/*}.rs` exist; `component.rs:24` `STATE_PRECEDENCE`; `button.rs:93` `shows_loading`; `input.rs:53` `resolve_chrome`; `theme.rs:15-23` `EGUI_*` are `const` (private); `semantic.rs:290` light/dark test; `alert/handler.rs:26-44` uses `theme.semantic.status.*`; `interact.rs:11` focus ring `STROKE_THICK`+`border_focus` |
| native-ui-component-core | n/a — directory contains only empty `evidence/`; no PLAN/CHECKLIST | Untracked orphan dir (not in git; `git status` clean) | STALE-DOC (housekeeping: remove dir) | `ls docs/plans/active/native-ui-component-core/` → only empty `evidence/`; `git log` for path returns nothing |
| core-component-style-system | IMPLEMENTING, first batch | Delivered claims verifiable in code; captures exist in plan evidence | SUPPORTS | VERIFICATION.md lists real commands/captures; tokens + component contracts present |
| native-core-ui-modernization | checklist all impl items `[x]`; verification traversal items unchecked | Primitives all exist in code; `reduce_motion` gate has test (checklist still unchecked) | SUPPORTS (STALE-DOC on verification checklist) | `overlay/toast.rs:255` ToastManager; `chrome/ui.rs:128` Skeleton; `feedback/ui.rs:111` Spinner, :8 Progress, :172 kbd_badge; `selection/ui.rs:148` Switch; `tabs/ui/segmented.rs:15` SegmentedTabs; `legacy.rs:864` status_dot; `animation.rs:27` `should_animate` + test :168 |
| native-shadcn-ui-system | All 5 phases done incl. gallery + screenshots | Component library exists; gallery redesigned since; evidence screenshots point to `/Users/truongdev/.gemini/...` paths outside repo | SUPPORTS with STALE-DOC | Components + `component_gallery_view.rs` exist; VERIFICATION evidence paths are non-repo absolute paths (NOT VERIFIED artifacts); plan text references "web frontend uses shadcn/ui" (archived) |
| component-layer-structure | RUNTIME_VERIFY; required 5-file contract | All 39 component dirs have mod.rs+config.rs+DESIGN/README/API.md; `common/` correctly exempt (shared infra); loose files = shared infra only | SUPPORTS (runtime a11y evidence still missing per own VERIFICATION) | `ls components/*/`: every dir ALL; loose = `animation.rs, common_utils.rs, disclosure.rs, interact.rs, legacy.rs, mod.rs` only; VERIFICATION "Runtime screenshot/accessibility evidence: NOT RUN" |
| component-interaction-polish | RUNTIME_VERIFY; impl delivered | Code supports claims | SUPPORTS (EVIDENCE_GAP: full heights + held-button recording) | `button/ui/button.rs` press-scale test `press_scales_background_icon_text_and_underline_together` (:461); `alert/ui.rs:129-143` dismiss at trailing edge; `text_selection_style.rs` recolor + tests :126,:202; checklist: 2 items still `[ ]` |
| component-gallery-redesign | RUNTIME_VERIFY; vertical rail, one category at a time | Code matches; captures exist for 3 widths (heights clamped at 1440/1920) | SUPPORTS (EVIDENCE_GAP: full 900/1080 heights) | `component_gallery_view.rs:340-405` 196px rail, 15 categories; `:434-447` match renders only selected; capture env `DB_PRO_CAPTURE_COMPONENT_GALLERY`/`_GALLERY_SECTION` in `capture.rs:80` + `workspace_actions.rs:983` |
| gallery-layout-collisions | 4 fixes claimed, captures recorded | All 4 fixes present in code; evidence PNGs in plan dir | SUPPORTS | `explain/ui.rs:277+` reserves stat width; `database/ui.rs:143-146` accounts item_spacing+SPACE_XS; `workspace/ui.rs:47-81` measured right block + `…` overflow (:91,:102); `component_gallery_rendering.rs:167-168` alpha modulates `theme.text_primary` |
| gallery-form-controls | RUNTIME_VERIFY; fixes claimed done, manual cursor/focus pass open | Code fixes all present; **VERIFICATION.md is an empty stub** ("Work in progress") — no commands/captures recorded despite CHECKLIST `[x]` "quality gates and captures" | EVIDENCE_GAP + STALE-DOC | `input/layout.rs:29-40` precedence resolves single stroke; `alert/ui.rs:133-138` frameless dismiss `egui::Button .frame(false)`; `selection/ui.rs:139,267,378` PointingHand + cursor tests :573-580; VERIFICATION.md = 6 lines, no evidence |
| disclosure-components-polish | RUNTIME_VERIFY; shared disclosure family | Code supports; only 1 capture (dark 1280×800); hover/transition need interactive review | SUPPORTS (EVIDENCE_GAP: light theme, other viewports, animation) | `disclosure.rs:18` `disclosure_progress`, :68 `paint_header_surface` (transparent rest → hover/open fills), :94 focus ring, :123 `paint_chevron` crossfade, :60 `set_opacity(openness)` fade; used by `collapsible/ui.rs:149`, `accordion/ui.rs:148`; screenshots/ has 1 PNG |
| ui-component-language | Slices migrated to canonical components | Code confirms (in refactored files, not the original filenames) | SUPPORTS (EVIDENCE_GAP: screenshots blocked per own doc, pending #287) | `result_grid_inspector_surface_view.rs:3,85,89-102` SegmentedTabs+Button variants; `explorer_toolbar_view.rs:50` SearchInput, :32 `ctx_menu_item`; guard test `components/mod.rs:129` |
| ui-core-audit | P0s fixed #307; 5 P1 + P2 open | P0s verified in code. P1s: responsive width + overlay fade + focus indicator **now present in code** — doc lists them "still open" | SUPPORTS (P0) + STALE-DOC (P1 status) | `dialog/ui.rs:96,101,107` register/Esc/trap_focus; `modal_guard.rs:42,72` + tests :102-168; `handler.rs:38,65` Esc/backdrop tests; P1 now in code: `common/layout.rs:25` width clamp to screen, `dialog/ui.rs:129,162` fade_alpha+opacity, `interact.rs:7` paint_focus_ring used by select/tabs/selection/dialog; a11y `widget_info` partial (`table/ui.rs:76,301`, `tree/ui.rs:65`) — full a11y tree unverified |
| quick-open-action-visuals | RUNTIME_VERIFY; strokes removed, rings kept, captures exist | Code supports | SUPPORTS (EVIDENCE_GAP: full 900/1080 heights clamped) | `palette_surface_view.rs:22-27` selection.stroke.color + test :201; `:60-61` `bg_stroke = Stroke::NONE` for scope filters; `:96-120` selected row = fill only; `button/handlers/palette.rs:23-59` all action variants `stroke_*: Stroke::NONE`; `button/ui/button.rs:316` focus ring kept |

## 3. UI findings

**Color32 sweep (quantified).** `Color32::` outside `theme.rs`/`tokens/`: **109 occurrences across 37 files**. Breakdown:
- ~76 are structural constants (`TRANSPARENT`/`PLACEHOLDER`) or test-module code — acceptable.
- **~29 production literals**, concentrated:
  - `chart_view.rs:24-37` — **12 hardcoded chart series colors** (`#2563eb` etc.), a real palette duplicated outside the token system. **P1** (AGENTS.md: views must not invent colors; duplicates semantic status roles).
  - `component_gallery_rendering.rs` — 6 literals; ~half are intentional sample/backdrop fixtures (black-alpha shadow, `from_gray(96)` raster reference), `text_primary`-modulated alpha at :168 is correct. **P2** (fixture-scoped).
  - `dev_tools/mod.rs:311-317` — 3 debug-overlay colors (orange/red rect markers). **P2** (dev-only).
  - `alert/ui.rs:229,253`, `hover_card/ui.rs:147`, `legacy.rs:924` — `from_black_alpha` shadow/backdrop literals; consistent pattern, but bypass semantic overlay tokens. **P2**.
  - `diagram_view.rs:671,685,950`, `tree/ui.rs:219`, `toggle/ui.rs:242`, `command/ui.rs:201` — `Color32::WHITE` used for **text-measurement galleys only** (width math, never painted). Harmless but worth a named constant. **P2**.
  - `animation.rs:137` `from_black_alpha` overlay modulation — inherits overlay alpha. **P2**.
- `selection/ui.rs` PLACEHOLDER galley colors (:131,:135,:254,:259,:263,:370,:374): text galleys painted with `Color32::PLACEHOLDER` sentinel — verify no visible placeholder-color text at runtime. **P2**.

**Duplicate tokens:** no duplicated `SPACE_*`/`RADIUS_*` definitions found in views — migration removed component mirrors (per component-layer-structure VERIFICATION). `palette_surface_view.rs` uses raw literals for spacing/radius (`add_space(6.0/8.0/16.0)`, `Margin::symmetric(12,8)`, `CornerRadius::same(6.0)`, `44.0` row height, `580.0` width) instead of tokens — same file the quick-open plan just touched. **P2**.

`component_gallery_view.rs:347-348` hardcodes rail width `196.0` — P2.

## 4. Feature findings

- **P1** — `chart_view.rs` chart palette bypasses `DbProTheme`/semantic status roles entirely (12 literals); also theme-insensitive (same colors light+dark, unverified contrast).
- **P1** — `gallery-form-controls` VERIFICATION.md is an empty stub while CHECKLIST marks gates/captures done; the plan's own acceptance ("captures recorded in VERIFICATION.md") is unmet. Only the manual-cursor item is honestly open; the file must record actual commands/captures or the [x] is false.
- **P2** — `ui-core-audit` doc claims 5 P1s "still open," but code shows responsive width (`common/layout.rs:25`), overlay fade (`dialog/ui.rs:129-162`), and focus indicator (`interact.rs`, used in ≥5 widget families) are implemented; a11y tree coverage is partial (table/tree/dialog only) — doc under-reports current state.
- **P2** — `native-ui-component-core` is an empty untracked plan dir (no docs at all); should be removed or populated.
- **P2** — `native-shadcn-ui-system` VERIFICATION cites screenshots at `/Users/truongdev/.gemini/...` — artifacts not in repo, unverifiable; gallery design since superseded by component-gallery-redesign (rail replaces the "category tabs + All" this plan shipped).
- **P2** — `legacy.rs` (43 pub fns) still hosts `spinner`/`status_dot` used by `monitoring_header_view.rs:64`; ui-component-language defers removal — consistent, but it remains a second convention surface.
- **P2** — No automated runtime evidence anywhere for disclosure hover/chevron animation, button press animation (recorded honestly as open in both plans).
- **P2** — `common/` component dir lacks config/DESIGN/README/API but is deliberately exempt per plan (shared infra) — no finding, noted for completeness.
- Test-gap note: `paint_field_chrome` precedence is unit-tested via `resolve_chrome` (`input.rs:72-84`) — OK. Cursor behavior only has `hovered_cursor` unit test; OS cursor verification remains manual per plan.

## 5. Captures taken

**None.** `target/debug/db-pro-native` and `target/release/db-pro-native` were both built **without** the `capture` cargo feature (`strings` shows `DB_PRO_CAPTURE_COMPONENT_GALLERY` absent from both; `capture.rs` is `#[cfg(feature = "capture")]`, native-app Cargo.toml:17). A `DB_PRO_CAPTURE_COMPONENT_GALLERY=1 DB_PRO_CAPTURE_TO=...` run of the debug binary produced no PNG in 60s. Rebuilding with `--features capture` was out of scope (no workspace builds allowed; 5 sibling agents running). Existing plan-dir screenshots were used for evidence instead.

## 6. Missing evidence to reach COMPLETED (per plan)

- theme-core-token: native captures at 1280×800/1440×900/1920×1080 of token-consuming surfaces (self-declared open).
- native-ui-component-core: n/a — dir is empty; decide delete vs populate.
- core-component-style-system: loading/error/empty state traversal (per VERIFICATION); light-theme pass of affected surfaces.
- native-core-ui-modernization: manual light+dark traversal; VERIFICATION.md never written.
- native-shadcn-ui-system: repo-resident evidence artifacts (current ones point outside repo); doc refresh vs gallery-redesign.
- compo

… [truncated for memory entry; see source file]

