# Layout Aesthetics Audit — Report

**Partial.** Scored 2026-10-23 by nine per-surface audit agents (one per feature) against `RUBRIC.md`
(eight dimensions, 0–4, 32 max). Scores are expert judgement with anchors and cited measurements,
not user testing.

**Evidence:** `/private/tmp/dbpro-audit/evidence/<surface>-<theme>-<WxH>.png`, six cells per surface
(1280×800, 1440×900, 1920×1080 × light/dark). 1440/1920 render at **838 logical height** on this
host, so vertical composition above 838 pt is unverified for every surface.


## Independent scoring pass (second reviewer) — full matrix, SHA 65fad16d

All 60 cells re-captured at one generation (manifest `evidence/manifest.json`, per-cell sha256) and
independently scored by five reviewers who did not read this report. Numbers differ deliberately —
disagreements are recorded, not averaged.

| Surface | Pass 1 (light/dark) | Pass 2 (light/dark) | Δ | Adjudication |
|---|---|---|---|---|
| Welcome | 15 / 17 | 25 / 25 | +10 / +8 | Pass 1 was pre-ladder (cards 1.08:1 vs canvas, now raised) and pre-F7b (selection pill); pass 2 scores the fixed theme. 1920 4-col text imbalance (new, P3) |
| Query | 17 / 17 | 23 / 24 | +6 / +7 | Post-fix toolbar and chevron order; pass 2 docked light for weak active-tab contrast (P2, new) |
| Table | **n/a — pass 1 never rendered** | 21 / 22 | n/a | Pass 1 scored from source only (its capture leaked Welcome); Table carries a single independent number, an admitted gap, not a fabricated second |
| Explorer | 20 / 20 | 24 / 24 | +4 / +4 | Selection pill now visible; popover content-edge bleed noted P3 |
| Settings | 18 / 20 | 25 / 25 | +7 / +5 | Rail pill + raised card fixed; pass 2 docked for ~60% dead space (P2, new) |
| Connection | 14 / 16 | 24 / 24 | +10 / +8 | Field boundaries added after pass 1; pass 2 flagged column-drift grid (P2) and the at-rest clipped disclosure (resolved as scroll affordance, F8) |
| Gallery | 18 / 20 | 21 / 21 | +3 / +1 | Selection visible now; docked for ~60-80% dead space (P2) |
| ER diagram | 14 / 13 | 23 / 23 | +9 / +10 | Zoom/card fixes landed post-pass-1; docked for decorative accent use on node headers (P2, new) |
| Agent | 19 / 19 | 24 / 24 | +5 / +5 | Composer/panel fill gap resolved; muted suggestion text borderline (P3) |
| Palette | 17 / 19 | 24 / 25 | +7 / +6 | Exact-match ranking + row contrast fixed; light muted action text ~3:1 (P2, new) |

Pass-2 mean ≈ **23.4/32** vs pass-1 mean ≈ 17.4/32 — the +6 reflects fixes landed during the audit,
not scorer leniency; every delta ≥ 2 dimension is accounted for above.

## Scores

| Surface | Light | Dark | Note |
|---|---|---|---|
| Welcome + shell chrome | 15/32 | 17/32 | light loses 2 on colour (cards 1.08:1); rail selection ΔL 0 was fixed by F7b after scoring |
| Query workspace | 17/32 | 17/32 | strongest surface; loses on type scale (5 sizes) and icon sizes (4) |
| Table (data grid) | — | — | **source-derived only**: the first capture run rendered Welcome; the fresh grid capture exists but has not been re-scored |
| Explorer | 20/32 | 20/32 | best surface; unchecked control boundary 1.26:1 |
| Settings | 18/32 | 20/32 | *pre-fix score* for the rail; light input still has no boundary (open) |
| Connection dialog | 14/32 | 16/32 | 4 column seams in one 772 pt form; 7 type sizes |
| Component gallery | 18/32 | 20/32 | *pre-fix score*: light selection was invisible; fixed by F7b, not re-scored |
| ER diagram | 14/32 | 13/32 | opens at 41–44 % zoom, cards render as name strips |
| Agent panel | 19/32 | 19/32 | composer fill equals panel fill |
| Command palette | 17/32 | 19/32 | 7 type sizes; selection 1.08–1.13:1 |

Mean ≈ **17.4/32**. Nothing reaches 24/32 (3/4 average). Dimension 5 (colour & surface) is the
weakest across the board — it scores 1–2 on seven of nine surfaces, and it is the dimension that
decides whether the product looks like a database tool or a prototype.

## What the numbers say (recurring, not per-surface)

1. **The light theme has no usable elevation ladder.** `canvas = elevated = editor = floating =
   #ffffff` and `panel = #f6f6f6`, so cards (1.08:1), inputs (1.0:1 with no stroke), the command
   field (1.13:1) and popovers all separate by a hairline at best. Dark uses four distinct steps and
   scores 2–3 higher on this dimension everywhere.
2. *(Fixed during the audit — kept because it explains the pre-fix scores.)* **Selection was carried by fill in light, and the fill collides.** `surface_2` (Secondary rest) =
   `background.subtle` = `#f6f6f6` = `surface_panel`, so a selected rail/gallery row is pixel-identical
   to its background. Dark gets away with it by accident (Δ21).
3. **Type scales drift upward in count, not size.** Query 5 sizes, Settings 4, Welcome 6, Connection 7,
   Palette 7, Explorer 6, Gallery 4. Several are off-token literals (12.5, 10.5, 10.0, 13.5, 12.5).
4. **Icon optical sizes multiply.** Welcome 5, Query 4, Palette 2, Explorer 5 — one family, several
   sizes, no role mapping.
5. **Layouts do not distribute at width.** Query toolbar is 62 % empty strip at 1920; Welcome's grid is
   fixed at 620 pt (13.6 % coverage at 1920); gallery card fills 48 % of its column; diagram coverage
   falls to 13.6 %.

## Fixed during the audit

| Ref | Fix | Evidence |
|---|---|---|
| F7a | Explorer filter popover content inset (8 pt, component-layer `popup_content_frame`) | 6 Explorer cells, re-inspected |
| F7b | Selected rows marked by the shared accent pill (`paint_selection_pill`): activity rail, sidebar rows, gallery nav, settings rail | 6 Welcome cells + gallery/settings 1280 re-inspected; the rest of those surfaces are not yet re-opened |

## Not covered

- After the selected-nav pill change, Settings (3 of 6 cells) and Gallery (5 of 6 cells) were not
  re-inspected at every width/theme; their scores predate the pill and are superseded by the pass-2 table above.

- Table data grid: pass 1 had to score from source only (density 3/4, alignment 2/4, typography 1/4,
  colour 2/4, consistency 2/4, composition `n/a`) because its first capture rendered Welcome; the
  pass-2 number (21/22 light, 22/32 dark) is the only complete score, so the surface has one
  independent number rather than two.

- Declared sub-surfaces: Table structure, Settings → Appearance, Connection **error** state.
- Real states (empty/loading/error) per surface.
- Independent second scoring pass.
- Heights above 838 logical.

## Next

The five recurring items above are the audit's actual output. Each maps to an owning plan in
`FINDINGS.md`; nothing here proposes a redesign.
