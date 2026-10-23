# Layout Aesthetics Audit — Findings

Baseline: `493e3b9b`. Scores live in `REPORT.md`; this file holds the issues themselves.

## Fixed during the audit

### F7a: Explorer filter popover content had no inset (P2, owner-reported "siêu xấu")
- **Surface:** Explorer toolbar → object-filter popover, both themes.
- **Symptom:** Every row, the separator and the `Reset` link sat 4 pt from the popover's rounded edge
  (`menu_margin = SPACE_XS`), so the separator bled edge to edge, the section headers read as ordinary
  rows, and the block looked like an unstyled form rather than a menu.
- **Fix:** the popup body is wrapped in a margin-only frame (`SPACE_SM` inset, no fill or stroke — the
  popup frame still owns the surface), row spacing tightened to `SPACE_XXS`, and `SPACE_SM` added
  above the separator, above "Object kinds" and above `Reset`.
- **Evidence:** `evidence/explorer-{light,dark}-{1280x800,1440x900,1920x1080}.png` — all six cells
  captured from the fixed tree and inspected.

## Open observations (need scoring, not yet rated)

| Ref | Observation | Surface |
|---|---|---|
| A1 | In light mode the Welcome cards are white on a white canvas separated only by a hairline border, while the same cards in dark mode use an elevated fill — light relies on borders alone for elevation | Welcome / Explorer |
| A2 | The picker's dense state shows five rows in a 180 pt window with scrollbars hidden by the component contract, so a long list has no visible "more below" affordance | Query picker |
| A4 | The status bar is a single low-contrast run of six segments across the full width; it reads as a caption rather than a band | All workspaces |
| A5 | Selection glyphs are 16 pt (`CONTROL_SIZE = SPACE_LG`) against ~11–13 pt labels, so indicators dominate the row; this is component-wide, not popover-specific | All surfaces with `Checkbox`/`Radio` |
| A6 | At 1920 the Welcome cluster stays left-of-centre with a large empty right side; the composition does not recentre or scale | Welcome |

A5 belongs to `core-component-style-system` (component geometry), A1/A4/A6 to
`native-visual-redesign` (composition and elevation), A2 to this audit's own follow-up because the
scroll affordance is a picker decision.

## Scoring

Not started. Scores land in `REPORT.md` once the matrix is measured.
