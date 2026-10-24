---
schema_version: 1
id: "MEM-SRC-61bf80fe670193a9"
title: "Source mirror: docs/plans/active/ui-consistency-wave/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-consistency-wave/PLAN.md"
source_sha256: "61bf80fe670193a994f8b5cbface5c98815caa1787c04b56f6f02fb8bd2a482a"
mirror_body_sha256: "61bf80fe670193a994f8b5cbface5c98815caa1787c04b56f6f02fb8bd2a482a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# UI consistency wave

State: REVIEW. Baseline SHA: `89095cc7d4b056a9fcc19dd9368f7c2863c8759f`. Fix commit:
`0e370498`; gates and runtime captures recorded in `VERIFICATION.md`.

## Scope

Fix the systemic UI consistency findings from the two audit rounds (see `REPORT.md`):
the shared Select contract, affected call sites, the three P1 layout defects, and the
concrete report-1 layout/state findings that share the same components.

## Acceptance

- P1: explorer Refresh no longer overlaps Filter; agent composer renders top-down;
  schema picker scrolls and is keyboard-reachable.
- Select: no layout re-anchoring, no light-theme default, disabled visual, draft
  arrow selection with Escape revert, option semantics, single-row label contract,
  menu rows truncate instead of overflow.
- Call sites: one height per row context, widths from tokens, row caps localized,
  sidebar filters right-aligned, engine select themed in dark.
- Runtime evidence: light + dark at 1280×800, 1440×900, 1920×1080 for affected
  surfaces, including an open Select menu.
- Gates: fmt, check, clippy `-D warnings`, workspace tests, locked release build,
  `scripts/check-ui-architecture.sh`, clean-code diff ratchet.

## Non-goals

- Full ~417-string i18n sweep (tracked as UI-22; only strings touched by this wave are
  localized).
- Full dialog-width preset refactor (UI-20) — documented, not bundled with Select work.
- Provider/runtime behavior changes; no DB mutations.

## Plan docs

- `CHECKLIST.md` — execution checklist
- `FINDINGS.md` — severity and root-cause summary
- `VERIFICATION.md` — commands, counts, captures
- `REPORT.md` — consolidated audit report

## Wave 4 (2026-10-22) — Select width contract + workspace spacing rhythm

State: RUNTIME_VERIFY. Baseline SHA: `f984385a`.

Owner request: Selects whose content is tiny render absurdly wide and then fail to show
that content fully; re-measure how many places actually render UI fused together.

Scope:

- `Select` trigger width becomes content-hugging unless `.width()` pins it; the popup
  sizes from the measured widest option and pins itself with `ui.set_width`.
- `shell_frame_view.rs` stops zeroing `item_spacing` and carries the theme rhythm
  (`SPACE_SM × SPACE_XS`) so a view cannot silently render flush.
- Six icon-glyph + caption-label rows move from 0–2 px to `ICON_TEXT_GAP`.
- The audit skill gains a `ZERO-SPACING` static rule, a measured `fusion_scan.py`
  runtime scanner, two heuristics, and a variant-safe capture filename.

Out of scope: the pre-existing Schema Compare clipped diff list (UI-30) and the shared
`ScrollArea`'s always-hidden scroll bar (UI-31) are filed, not fixed.

Acceptance:

- Row-limit popup measures ≈ option width, not screen width, with no ellipsized option.
- Full re-verify matrix: 17 surfaces × 3 sizes (1280×800, 1440×900, 1920×1080) ×
  dark + light, checked for over-spaced regressions after the shell flip.
- Fusion counts come from painted geometry with a pixel crop behind every claim.
- Captures never open a window on the owner's desktop (Xvfb, and capture mode now
  creates an unmapped X11 window).
