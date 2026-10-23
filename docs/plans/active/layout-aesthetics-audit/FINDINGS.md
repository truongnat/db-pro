# Layout Aesthetics Audit — Findings

Baseline: `493e3b9b`. Findings accumulate as the audit runs; nothing here is a score yet.

## Inherited context (already fixed, kept as the starting point)

| Ref | What changed at `457cd645` | Why it matters to this audit |
|---|---|---|
| F1 | Horizontal `Radio`/`Checkbox` allocate content width and stay inside the parent | Removes the clipped "Dark" label that distorted the Appearance composition |
| F2 | Query context picker bounded to 300–360 pt with row-sized list regions | Removes the ~600 pt slab and the dead space that dominated the editor |
| F3 | Distinct `ScrollArea` ids in the picker | Removes debug banners painted over rows |
| F7 | Light chrome one step below the canvas (`#f6f6f6`), ladder canvas → panel → hover → selected | Gives the light theme readable regions; hover and selection stay distinguishable |

These are correctness/legibility fixes, not an aesthetic verdict. The audit re-judges them on the
new evidence rather than assuming the result reads well.

## Open observations (to verify with captures, not yet scored)

| Ref | Observation | Needs |
|---|---|---|
| A1 | Light chrome and card surfaces are now distinct, but cards on the chrome may read as one flat
sheet where borders are the only separation | Light/dark captures of Settings, Welcome and Explorer with cards on chrome |
| A2 | The picker's dense state shows five rows in a 180 pt window with scrollbars hidden by the
component contract, so a long list has no visible "more below" affordance | Dense-list capture plus a judgement on discoverability |
| A3 | Explorer filter popover rows are tight (label and first radio nearly touch) compared with the
checkbox block below | Popover capture at both themes; measure row gaps |
| A4 | Sidebar and status bar share the chrome fill; the status bar may read as an extension of the
sidebar rather than a separate band | Workspace captures; compare band heights and separators |

## Scoring

Not started. Scores land in `REPORT.md` once the matrix is captured and measured.
