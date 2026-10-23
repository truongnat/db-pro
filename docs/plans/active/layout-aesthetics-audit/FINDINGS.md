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
