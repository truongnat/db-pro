---
schema_version: 1
id: "MEM-SRC-2a541c037728244d"
title: "Source mirror: docs/plans/active/ui-consistency-wave/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-consistency-wave/FINDINGS.md"
source_sha256: "2a541c037728244da1aff4b362df095acbb9e713ff25473e7b043809b38ae418"
mirror_body_sha256: "2a541c037728244da1aff4b362df095acbb9e713ff25473e7b043809b38ae418"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — UI consistency wave

Consolidated detail: `REPORT.md`. Persistent audit memory:
`.skills/ui-consistency-audit/references/findings-ledger.md`.

## Counts at baseline `89095cc7`

- P0: 0
- P1: 3 (UI-01 explorer toolbar overlap, UI-02 composer order, UI-03 schema picker
  reachability)
- P2: 22

## Root causes

1. `Select::show` re-anchors itself with `with_layout(top_down(Align::Min))`
   (`select.rs:170`) and defaults to the light theme (`select.rs:83`).
2. Component state coverage is incomplete: disabled, error/required, option
   semantics, Escape-revert.
3. Container layouts are hand-rolled and nest RTL/LTR incorrectly (explorer toolbar,
   history filter) or reverse authored order (composer `bottom_up`).
4. Strings, sizes, and widths are duplicated as literals instead of tokens/locales.

## Resolution (commit `0e370498` on top of `89095cc7`)

- P1: 3/3 fixed (explorer toolbar, composer order, schema picker scroll).
- P2: 20/22 fixed; UI-20 (dialog width presets + destructive footer) and UI-22
  (~417-string i18n sweep) deferred with rationale in `VERIFICATION.md`.
- Evidence: 42 runtime captures in `/tmp/ui-audit/wave2/`; gates recorded in
  `VERIFICATION.md`. No new P0/P1 introduced.

## Baseline vs regression

No regression is introduced by this wave; every finding is inherited from the baseline.
Verification must re-measure the same geometry and report inherited issues that remain.

## Wave 4 counts (baseline `f984385a`)

- P0: 0
- P1: 2 fixed (UI-26 Select popup width, UI-27 shell zeroed `item_spacing`) + 1 open
  pre-existing (UI-30 Schema Compare diff list unreachable)
- P2: 2 fixed (UI-28 six fused icon+label rows, UI-29 gallery separator stub) + 1 open
  (UI-31 shared `ScrollArea` hides its scroll bar)

### How many places actually render fused

| Measurement | Before | After |
|---|---|---|
| Static `ZERO-SPACING` candidates (≥2 children, no spacing call) | 142 | 148 — advisory only: the shell no longer zeroes spacing, so a hit now means "no explicit gap here", not "fused" |
| Runtime `fusion_scan.py` over 17 surfaces @ 1440×900 dark | 20 raw pairs → 17 shared SidePanel edges + **3 confirmed fusions** | n/a |
| Runtime `fusion_scan.py` over the full matrix (17 surfaces × 3 sizes × dark+light = 102 dumps) | n/a | 142 raw pairs → 130 panel edges + 12 × one tree-row pair disproved by pixel measurement → **0 confirmed fusions** |
| Icon-glyph + caption-label rows at 0–2 px (grep + crop) | **6** → all moved to `ICON_TEXT_GAP` | 0 |
| Product audit rules (`overlap.suspicious`, `visual.touching_controls`) | 0 issues | 0 issues (10 011 overlap pairs evaluated on the results surface alone) |

So the honest answer to "how many places are fused": **9 real ones** (3 container-level
from the zeroed shell + 6 icon/label rows), all fixed; the remaining scanner hits are
panel edges and rect-padding artifacts, each checked against painted pixels.

### Root causes

1. `Select` sized its trigger from `ui.available_width()`, floored the menu at a magic
   `MENU_MIN_WIDTH = 152`, and left the popup unpinned while its rows self-reported the
   screen width — three independent defects that all had to go for the popup to hug content.
2. egui propagates `item_spacing` into `Frame`/`ScrollArea`/`horizontal`/`vertical`; the
   workspace `CentralPanel` set it to `ZERO`, so every view that forgot an explicit gap
   inherited flush rendering. Views compensated locally instead of the shell being fixed.
3. A lucide glyph painted as a `ui.label` fills its em box, so `item_spacing.x` of 0–2 px
   puts the next label's ink inside the glyph strokes.
