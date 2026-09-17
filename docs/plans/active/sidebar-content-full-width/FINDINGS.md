# Findings — sidebar-content-full-width

## P0 / P1

None open for this fix scope after the header hover fix.

## Evidence (pre-fix)

1. Screenshot crop: drag line ~445px, `New query` chrome ~360px, tree/`Failed:` ink ~200px.
   Tree was not filling the sidebar content width.
2. Workspace selector painted `surface_hover` *after* the label → hover wiped the name
   to a blank wash ("trắng xóa").

## Fixes in this branch

- Content rect + drag line locked to `sidebar_width`
- Explorer ScrollArea / tree rows forced to content width
- Header selector: allocate → hover wash → text (correct paint order)
- Removed duplicate Plus on the explorer filter row (New Connection stays in header)

## Residual

Runtime screenshot evidence still pending after native rebuild.

---

## Follow-up findings (2026-09-17)

Reported: the tree is still overlapped by the width — the `Native Test` row paints a badge
but it is hidden — and the filter field's border is missing both of its ends.

### P1 — tree rows overflow the column and lose their trailing badge

Evidence (pre-fix, 1440×900, `sidebar_width = 260`):

```text
TOOLBAR row_width=240 search_width=204 spacing=8 btn=[[220.0 107.0] - [254.0 131.0]]
ROW max_rect=[[8.0 146.0] - [290.0 892.0]] avail=282.0 -> width=282
#33 Rect fill=#EE_EE_EE bounds=[[239.0 155.0] - [286.0 171.0]] clip=[[8.0 150.0] - [248.0 176.0]]
#34 Text "SQLITE"        bounds=[[247.0 159.5] - [278.0 167.5]]
```

The row is allocated 282 wide in a 240 column. The clip keeps only `x 239–248` of the pill,
so the label survives as a ~1px sliver: the badge is *painted* and *invisible* at once,
which is exactly what was reported.

Severity **P1**: a value the product deliberately surfaces — the connection's driver — is
unreadable. Not P0: no data, mutation or transaction is involved.

**Runtime verified** — the screenshots in `screenshots/` (1280×800 / 1440×900 / 1920×1080)
show the `SQLITE` and `PG` driver badges fully readable, the pill ending inside the clip with
the label fully painted (31px of text in a 39px pill).

Provider impact: none. This is layout only; PostgreSQL and SQLite render the same tree.

### P1 — every 1px border on a widget filling the column loses both vertical edges

Evidence (pre-fix): `#29 Rect stroke width=1.0 rounding=6 bounds=[[7.5 101.5] - [248.5 136.5]]`
against `clip=[[8.0 8.0] - [248.0 892.0]]`. The stroked path is `[8,248]`; an outside stroke
paints `[7,249]`, so the clip removes both vertical bands entirely while leaving the
horizontal ones, which sit well inside it.

Same defect and root cause, measured on widgets the report did not name:
`#13 Rect stroke bounds=[[7.5 47.5] - [248.5 76.5]]` — the `New query` primary button.

Severity **P1**: a primary action's outline is missing two of four edges.

### Disproved along the way

- **epaint tessellates four-edged rounded-rect strokes correctly.** Rasterizing the
  tessellator's own `rect_stroke` output for a 240×35 rect filled all four edges and all
  corner arcs. The loss was clipping, never tessellation.
- **A degenerate or very thin rect was not the cause.** The pre-fix field was 240×34.
- **The row-level clip clamp was written, measured, and removed.** `ScrollArea` narrows its
  clip by the scrollbar (`inner_size = outer_size - current_bar_use`,
  `scroll_area.rs:555`), so a row clamped to the clip shrank to 230 whenever the list grew
  long enough to scroll — and back to 240 when it did not. The bound moved to `tree_width`,
  before the `ScrollArea`, and `the_tree_row_spans_its_layout_width_not_its_clip` now pins
  the row to its layout width.

### Residual (P2)

- `SearchInput` still reserves guessed `36.0` / `20.0` for the shortcut badge and the clear
  button. Harmless now that the frame is capped, but it narrows the text area by roughly
  14px against the measured widths.
- The outside-stroke loss affects any 1px `Frame` / `Button` border flush with a container
  edge elsewhere in the app. Only the sidebar column carries the 1px bleed.
