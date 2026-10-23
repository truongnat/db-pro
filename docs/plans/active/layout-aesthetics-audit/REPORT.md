# Layout Aesthetics Audit — Report

Scored 2026-10-23 by nine per-surface audit agents (one per feature) against `RUBRIC.md`
(eight dimensions, 0–4, 32 max). Scores are expert judgement with anchors and cited measurements,
not user testing.

**Evidence:** `/private/tmp/dbpro-audit/evidence/<surface>-<theme>-<WxH>.png`, six cells per surface
(1280×800, 1440×900, 1920×1080 × light/dark). 1440/1920 render at **838 logical height** on this
host, so vertical composition above 838 pt is unverified for every surface.

## Scores

| Surface | Light | Dark | Note |
|---|---|---|---|
| Welcome + shell chrome | 15/32 | 17/32 | light loses 2 on colour (cards 1.08:1, rail selection ΔL 0) |
| Query workspace | 17/32 | 17/32 | strongest surface; loses on type scale (5 sizes) and icon sizes (4) |
| Table (data grid) | — | — | **source-derived only**: the first capture run rendered Welcome; the fresh grid capture exists but has not been re-scored |
| Explorer | 20/32 | 20/32 | best surface; unchecked control boundary 1.26:1 |
| Settings | 18/32 | 20/32 | light input has no boundary at all |
| Connection dialog | 14/32 | 16/32 | 4 column seams in one 772 pt form; 7 type sizes |
| Component gallery | 18/32 | 20/32 | light selection invisible (fixed here) |
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
2. **Selection is carried by fill in light, and the fill collides.** `surface_2` (Secondary rest) =
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
| F7b | Selected nav rows marked by the shared accent pill (`paint_selection_pill`), used by the sidebar, the gallery nav and the settings rail | gallery/settings cells re-captured |

## Not covered

- After the selected-nav pill change, Settings (3 of 6 cells) and Gallery (5 of 6 cells) were not
  re-inspected at every width/theme; their scores predate the pill.

- Table data grid: scored from source only (density 3/4, alignment 2/4, typography 1/4, colour 2/4,
  consistency 2/4, composition `n/a`); the fresh capture has not been re-scored, so no total is claimed.

- Declared sub-surfaces: Table structure, Settings → Appearance, Connection **error** state.
- Real states (empty/loading/error) per surface.
- Independent second scoring pass.
- Heights above 838 logical.

## Next

The five recurring items above are the audit's actual output. Each maps to an owning plan in
`FINDINGS.md`; nothing here proposes a redesign.
