# Layout Aesthetics Audit — Findings

Baseline: `493e3b9b`. Scores live in `REPORT.md`; this file holds the issues themselves.

## Fixed during the audit

### F7b: Selected rows were invisible in the light theme (P1, found by three agents)
- **Surfaces:** activity rail, sidebar rows, gallery category nav, settings section rail.
- **Symptom:** `ButtonVariant::Secondary` rest fill is `surface_2` = `background.subtle` = `#f6f6f6`,
  which equals `surface_panel` — the fill of every rail — so the active row was pixel-identical to its
  background in light mode. Dark escaped by accident (Δ21).
- **Rejected fix:** reordering `Secondary`'s fill ladder. `palette.rs` documents that hover must not use
  the selected role, and `Secondary`/`surface_2` also serve ordinary secondary actions and chips.
- **Fix:** selection is marked by the shared accent pill (`nav::paint_selection_pill`), used by
  `sidebar_item`, the gallery category nav, the settings section rail and the activity rail icon button.
- **Evidence:** Welcome 6/6 cells re-inspected after the change; gallery/settings 1280 cells
  re-inspected. The remaining gallery/settings cells were re-captured but not re-opened, so those
  scores stay annotated as pre-fix.

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

## Cursor affordance audit (owner-reported: cursor does not match behaviour)

**Evidence level: source only.** This inventory comes from reading every `CursorIcon` site and every
`Sense::click`/`Sense::hover` allocation; no runtime hover was driven yet, so each gap below is a
source-derived expectation, not an observed mismatch. Runtime confirmation is a checklist item.

Inventory: 51 `CursorIcon` sites in `crates/ui` + `crates/app` — 33 `PointingHand`, 12 resize
variants, `Grab`/`Grabbing`/`Move` in the diagram canvas, `Wait` on a busy button, `NotAllowed` on a
disabled select option, one `Default`. **`CursorIcon::Text` is never used anywhere.**

The convention this codebase already follows, stated once so the gaps are obvious:

| Element | Cursor |
|---|---|
| Button, link, tab, menu item, row that navigates or selects | `PointingHand` |
| Editable or selectable text | `Text` |
| Splitter / column border / dock grip | `ResizeHorizontal` · `ResizeVertical` |
| Window edge / corner | `Resize*` in the matching direction |
| Canvas pan / drag / movable object | `Grab` · `Grabbing` · `Move` |
| Disabled control | `NotAllowed` (or `Default`) |
| Busy | `Wait` |

### Gaps (clickable, but cursor stays the default arrow)

| Ref | Element | File | Expected |
|---|---|---|---|
| C1 | Result-grid cell — double-click opens the inline editor | `result_grid/result_grid_cell.rs:48` | `Text` on editable cells |
| C2 | Result-grid row gutter — click selects the row | `result_grid/result_grid_row_gutter_view.rs:11` | `PointingHand` |
| C3 | Query status-bar labels — toggle transaction, edit bind parameters | `query_workbench/query_status_bar_surface_view.rs:133,148` | `PointingHand` |
| C4 | Explorer tree rows — click selects, disclosure toggles | `explorer/explorer_tree.rs:98` | `PointingHand` |
| C5 | Tools-hub cards | `tools/tools_hub_surface_view.rs` | `PointingHand` |
| C6 | Table data toolbar clickable cells | `table/table_data_toolbar_surface_view.rs` | `PointingHand` |

### Judged acceptable

- Diagram: `Grab` on empty canvas, `Grabbing` while panning/dragging, `Move` over a node — the node
  case could arguably be `Grab`, but it is deliberate and consistent.
- `NotAllowed` on a disabled select option and `Wait` on a busy button match the table above.
- `PointingHand` on tabs, menu rows, tree rows in the core `Tree` component and select options is
  correct; the custom explorer tree simply does not inherit it (C4).

Routed to: `core-component-style-system` for C1/C2 (grid affordances live in the shared table/grid
components), and this audit's own follow-up for C3–C6 (view-level omissions).

## Wide-layout distribution (audit item 5)

| Ref | Surface | Before | After |
|---|---|---|---|
| W1 | Welcome grid | fixed 620 pt at every width (38.8 % at 1920, 13.6 % canvas coverage) | column count from the local width: 4 columns at 1920 (100 % of the content column, card 396 pt), 2 at 1440/1280, 1 below 572; `CARD_MAX_WIDTH = 420` keeps the reading measure |
| W2 | Gallery specimen card | 541 pt inside a 1120 pt column (48 %); header actions ~110 pt outside the column | card fills the column (`Card::fill_width`); header and body share one `detail_column` helper so both land on the same edges |
| W3 | `nav::section_label` | `SECTION_LABEL_FONT_SIZE = 10.5` literal | `FONT_SIZE_BADGE` — the last off-token label size is gone |
| W4 | Query toolbar | run/explain/format, then the row cap, then a ~60 % empty run to the pinned overflow | resolved by grouping, not by distributing: the row cap moved into the trailing group with the overflow action, so a wide strip reads "actions left, view settings right". The earlier equal-gap attempt was reverted because it clipped the cap label to "50…" |

W4 is closed as a grouping decision: an IDE toolbar with actions on the left and view settings on the right is the convention, and it keeps every control at its natural width. Distributing equal gaps across four anchored groups was tried and rejected — it required pinning the cap trigger to a width smaller than its label.

## Live (Orca-driven) verification — 2026-10-23

The four open items were checked against the running app, not the capture harness.

| Item | Result |
|---|---|
| Table **data** surface | `evidence/live/table-data-light-1280.png` — employees, 3 rows, grid header/cells/gutter legible, row separators visible |
| Table **structure** surface | `evidence/live/table-structure-light-1280.png` — header row, PK/FK badges, Actions column with `Alter` links |
| Settings → **Appearance** | `evidence/live/settings-appearance-light-1280.png` — rail pill on the active section, card on the canvas, Light/Dark radios adjacent and unclipped, checkbox bounded |
| Connection **error** state | `evidence/live/connection-error-light-1280.png` — every invalid field carries a red border and its own message, plus a toast |
| Cursor | **test-verified + one live hover datum**: `stale_cursor_is_cleared_before_widgets_draw` asserts both directions of the frame-level reset (clear-first keeps a widget's `PointingHand`; clear-after erases it). Screenshot proof of the pointer glyph was not obtainable here — the shared desktop kept another window frontmost. Later pass: `screencapture -C` proven working
(Terminal I-beam renders), frontmost confirmed `db-pro-native`, yet hovering `Connected` (status bar)
and the `Welcome` tab still showed the OS **arrow** rather than `PointingHand`. Unresolved whether the
hit-miss is a hit-test gap or the OS cursor request not reaching winit; `clear_stale_cursor` itself is
locked by `stale_cursor_is_cleared_before_widgets_draw` |

New finding from the live pass:

### F8 (RESOLVED — not a defect): Connection dialog's last section was clipped at rest, scrollable (P3 nit)
- **Evidence:** `connection-dark-1440x900.png` — the `Cloud Presets & URI Importer` disclosure is cut mid-row at the body viewport bottom.
- **Resolution (live Orca drive, 2026-10-23):** `DialogFrame::body` already wraps the form in
  `egui::ScrollArea::vertical().max_height(560)` (DIALOG_FORM_BODY_HEIGHT) with a pinned footer
  outside the scroll region. The a11y tree exposes a `scroll bar` element, and
  `orca computer scroll --element-index` moved the body so the Cloud Presets row renders
  fully above the footer (screenshot `live/connection-scrolled-light-1280.png`).
  The clipped appearance at rest is the standard "more below" affordance of a scroll viewport.
- **Remaining P3 nit:** no scrollbar is painted at rest because `VisibleWhenNeeded`'s
  opacity animation only rises on interaction; the half-clipped row is the only affordance.
  Recorded as polish, not blocking.

## Chevron order in the trailing group (fixed)

Moving the row-cap picker into the right-to-left group mirrored its trigger to `⌄ 500 rows`. The picker now draws inside a forced left-to-right layout, so it reads `500 rows ⌄` again (verified in the re-captured `query-light-1280x800.png`).

## Independent scoring pass (second reviewer) — **partial**

This pass scored **Settings at 1280 only**, with visual estimates rather than pixel measurements, and could not attribute the Query/1920 images (three capture generations in the directory). It does **not** close the independent-review item and does not validate the other eight surfaces.

| Surface | First pass | Independent pass | Disagreement |
|---|---|---|---|
| Settings (General) | 18/32 light · 20/32 dark | **22/32 light · 22/32 dark** | +4 light / +2 dark. The second reviewer scored colour 3/4 (the new ladder makes the card read as raised without a border) and composition 3/4; the first pass had scored colour 1/4 on the flat-white theme that no longer exists |

Disagreements are recorded, not averaged. The independent pass also flagged two things the first pass missed or under-weighted:

- **Primary action changes appearance across themes** (dimension 6, 2/4): `Save` is accent blue in light and a light-grey fill in dark, because dark `accent.solid` is achromatic by design. That is a deliberate token choice, but it means "primary" is not one visual role across themes — worth a decision, not a silent pass.
- **Light rail edge is faint** (dimension 5): the nav panel and canvas differ by few luminance steps, so the rail boundary is weaker than the card boundary.

Query workspace could not be independently scored in that pass: the evidence directory held files from three capture generations and the reviewer could not attribute them to labels. The four Query cells (light/dark × 1280/1920) were re-captured afterwards; a second pass on Query remains open.

## Findings from the independent pass (post-matrix, SHA 65fad16d)

### F9 (P2): scrollable regions hide every scrollbar — no "more content" affordance
`components::scroll_area::ScrollArea::build` hardcodes `ScrollBarVisibility::AlwaysHidden` for every
component ScrollArea. Consequences observed in evidence:

- `table-*-1280x800`: `created_at` timestamps clip mid-digit at the grid's right edge with no bar
  signalling horizontal scroll (the grid's ScrollArea does scroll — verified in source).
- `connection-*` (all 6): the last form section renders half-clipped at the pinned body's bottom —
  readable only after a scroll; the bar appears only after interaction.
- The dialog frame's own `VisibleWhenNeeded` (added earlier) still fades its bar unless the pointer
  interacts, so the same affordance gap remains even there.

Fix direction: expose `scroll_bar_visibility(...)` on the component wrapper (default `AlwaysHidden`)
and opt `VisibleWhenNeeded` on the grid's horizontal axis and any pinned-height body whose overflow
isn't otherwise advertised. Owner: `native-visual-redesign`.

### F10 (P2): Settings/General dead space
Settings card occupies the top ~40% of the canvas; ~60% dead space below at ≥1440×838. Content could
centre vertically or the card could grow; either is a design decision, not a bug.

### F11 (P2): New Connection form grid uses three different column splits
Name/Folder ≈55/45, Host/Port ≈72/28, Database/Username ≈48/52 — left-column right edges disagree by
~200px across rows. Unify to two column templates.

### F12 (P2): diagram node headers burn the accent role
10 decorative blue node headers + 1 teal chip on screen; only the chip is actionable. Mutes the
accent language elsewhere (same class of issue pass 1 flagged in other surfaces).

### F13 (P2): light-theme active-tab contrast too weak
Active query tab distinguished only by a faint top accent line; tab fill ≈ canvas fill.

### F14 (P3 cluster): recorded, no fix planned this cycle
Welcome 1920 card-copy line imbalance; palette light muted action text ~3:1; connection dialog opens
without a dim scrim; Save/Save Connection is accent blue in light but near-white in dark (same role,
different treatment); red TLS warning ~4:1 on dark; explorer popover content edges sit ~4pt from the
rounded border; stray floating panel-collapse icon bottom-left of sidebar; muted agent suggestion
text borderline 4.5:1.
