# UI consistency heuristics

Rules are named so findings, commits, and scanner output can cite them. Each rule has
a tell-tale pattern and a check. Add a rule here whenever a finding generalizes.

## LAYOUT-NEUTRAL

**A reusable control must not re-anchor itself inside `show`.**

Tell: `ui.with_layout(Layout::top_down(Align::Min), ...)` or unconditional
`ui.set_width(ui.available_width())` inside a component.

Consequences:

- top-aligns inside `ui.horizontal` (row of 28 px control vs 38/50 px siblings);
- left-anchors inside `Layout::right_to_left` (dead space on the right);
- ignores the parent's center alignment.

Check: measure sibling `center_y` in a horizontal row and `x_min` in an RTL row.
Fix: let the parent layout place the control; only allocate the trigger size.

## THEME-NO-LIGHT-DEFAULT

**A themed component must not default to the light theme.**

Tell: `theme: DbProTheme::default()` in a builder's `new`.

Consequences: one forgotten `.theme(...)` renders a white control on a dark dialog.
Light is the default `DbProTheme`, so this fails silently in light-mode screenshots.

Check: scanner `missing-theme`; review dark captures of every call site.

## STATE-COMPLETE

**Every interactive control paints: resting, hover, focus, open, disabled; forms add
required, error, helper.**

Tell: `add_enabled_ui` around a control whose paint path never reads `ui.is_enabled()`
or whose colors are unconditional.

Consequences: the control looks enabled while clicks are ignored.

## SIZE-RHYTHM

**Controls on one row share a height token; form controls match their input height.**

Known tokens: `INPUT_HEIGHT_DEFAULT` 38, `BUTTON_HEIGHT_SM` 28,
`BUTTON_ICON_SIZE_SM` 26.

Tell: `SelectSize::Sm` (28) beside `Input` (38) in the same form row; 38 input beside
26/28 buttons and an 18 px badge in a toolbar.

Check: measure `y_max - y_min` per sibling; compare with the token table.

## WIDTH-TOKEN

**Widths come from tokens, not literals.** Ten literal widths for one component
(88/96/104/108/120/140/160/180/220/260) is the failure signature: no two surfaces
agree, and a translation can overflow any of them.

Check: scanner reports literal `.width(<number>)`. Promote repeated values to
`tokens::component::*` constants.

## LABEL-CONTRACT

**Field labels use the shared `Label`/`FormField` contract (12 px strong,
`text_secondary`, required `*`), not hand-rolled `.small()`.**

Tell: `ui.label(RichText::new("X").small().color(theme.text_secondary))` immediately
above a control, or a component label painted with a different weight.

Consequence: two label styles in the same form.

## MENU-ROW-FIT

**A menu row must fit or truncate its label; it must not rely on wrapping inside a
fixed row height.**

Tell: `allocate_exact_size(..., TABLE_ROW_HEIGHT_COMPACT)` followed by
`painter.layout(..., wrap_width)`. Long options (driver names, file paths) paint past
the row.

Check: call the menu with the longest real option.

## A11Y-OPTIONS

**Painted menu rows need semantics; the trigger alone is not enough.**

Tell: `paint_option` allocates a `Response` but never calls `widget_info`; only the
trigger calls `combo_box_info`.

Consequence: a screen reader announces the combo box but no options. Also check
keyboard commit semantics: arrow keys must move a *draft* selection; Escape must
discard it, Enter/click commits.

## RTL-HONEST

**Right-aligned slots must actually be right-aligned.**

Tell: `ui.with_layout(Layout::right_to_left(...))` containing a component that
re-anchors top-down/left (see LAYOUT-NEUTRAL). Measured tell: control width 88 in a
164 px slot with 76 px of empty space on the right.

## I18N-NO-LITERAL

**No user-visible literal.** Row caps, empty states, footers, and menu headers all use
`t!`. Two controls of the same kind with different i18n behavior is a defect even if
both strings happen to be English today.

Check: diff call sites of the same component; one localized and one literal is the
tell.

## CAPTURE-DRIVABLE

**If a control owns open state, the capture harness must be able to open it.**

Tell: state stored in `ctx.data_mut().get_temp::<bool>(Id)` while the harness calls
`egui::Popup::open_id`. The hook silently does nothing and the audit has no menu
evidence.

## INLINE-ERROR-SOURCE

**A field's inline error and the dialog toast must come from the same validator.**

Tell: the capture/fixture path hardcodes an error message, or the view formats its own
required-field text while `validate_*` returns different wording. Two sources drift the
moment a field is added, and only one language reaches the toast.

Check: grep the validator's return strings against the `error_text` call sites; both
should be `t!` keys from the same module.

## STROKE-COUNTS

**egui `Frame` includes `2 * stroke.width` in its total size.**

Tell: a component computes `content_width = width - margin.left - margin.right` for a
bordered frame; the trigger ends up 2 px wider than its token box and sibling fields
drift. Subtract `stroke_extra` too, and assert the outer box in the field-geometry test
(`standard_field_geometry_…` covers the Select trigger).

## CONTRAST-COMPUTED

**Muted text is a measured ratio, not a palette opinion.**

Tell: a token is "obviously readable" until the relative luminance is computed on the
actual surface it is painted on. Add the computation as a unit test in
`crates/ui/src/theme/tokens/tests/` so a palette edit cannot silently drop below 4.5:1.

## SPACING-INHERITS-ZERO

**`ui.spacing_mut().item_spacing = ZERO` on a panel poisons every descendant container.**

Tell: text and controls rendered flush against each other ("dính sát") in one view while a
neighbouring view looks correctly spaced. egui propagates `item_spacing` into `Frame`,
`ScrollArea`, `ui.horizontal`/`vertical` children; only top-level `Panel`/`Area`/`Window`
reset it. `shell_frame_view.rs` used to zero the whole workspace CentralPanel, so every
view that forgot an explicit `add_space` fused its siblings — 142 candidate containers.
Fix at the root (theme rhythm `SPACE_SM × SPACE_XS`) and let views override locally to be
*tighter*, never rely on each view remembering to re-space. Scanner rule: `ZERO-SPACING`
(advisory; rect-positioned children are indistinguishable statically).

## POPUP-WIDTH-PINS

**A popup whose rows allocate `available_width()` must pin its width, not set a minimum.**

Tell: a Select menu measured 598 px wide for a 90 px option list, and no `auto_shrink`
setting changed it. Inside an `Area`, `available_width()` is the distance to the *screen*,
so `set_min_width` is ignored by content that self-reports that wide, and
`ScrollArea::auto_shrink([false, _])` happily fills it. Use `ui.set_width(...)` on the
surface and on the scroll content, and size the popup from the measured widest row
(`menu_row_width`) instead of a magic floor like `MENU_MIN_WIDTH = 152`.

## ICON-TEXT-GAP-COLLAPSES

**An icon glyph and its caption label need `ICON_TEXT_GAP`, not `SPACE_XXS`.**

Tell: "⚡Agent" or "⤓Schema & Migration" rendering with the glyph's descender
crossing the first letter. A lucide glyph is painted as a `ui.label` whose rect is
the em box, so `item_spacing.x` of 0–2 px puts the *next* label's ink inside the
glyph's visible strokes. The house value is `tokens::ICON_TEXT_GAP` (8 px) — the
same gap the agent suggestion rows and the Select trigger already use; measure a
correct row before inventing a new one. Grep for
`item_spacing.x = (SPACE_XXS|0.0|2.0)` and read the two widgets that follow.

## RUNTIME-FUSION-SCAN-BEATS-GREP

**Count collisions from painted geometry, not from `add_space` call sites.**

Tell: a grep audit reporting 74 "spacing collisions" of which most are fine, while
a real 598 px popup and six fused icon+label rows go unreported. Static scans can
only produce candidates (`ZERO-SPACING`). For measured counts, capture with
`capture_ui_matrix.sh --audit` (it sets `DB_PRO_INSPECTOR_DUMP`) and run
`scripts/fusion_scan.py <audit-dir>`: it parses the `WID` geometry lines and reports
same-parent sibling pairs with a sub-2.5 px horizontal gap. Two filters are
mandatory — full-height panel pairs (shared SidePanel edges are the contract) and
rect-gap-as-ink-gap (glyph rects include side bearings, so a 0 px rect gap can
still read fine; confirm with a pixel crop before filing).

## FALSE POSITIVES (do not re-report)

- Hidden focus anchors in dialogs (`dialog/ui.rs`): intentional, not zero-size bugs.
- A component label with `text_secondary` is not low contrast by itself; compute the
  ratio against the actual surface token first.
- `Sense::click()` in egui is focusable; "not keyboard accessible" requires checking
  the key handling, not the sense.
- Compact 28 px controls in toolbars are intentional; only mixed heights in the same
  row are defects.
- Fixture screens (`DB_PRO_CAPTURE_*` deliberately broken) are scanner tests, not
  product surfaces.
- A left-aligned control inside a left-to-right row is fine; RTL only matters when
  the parent explicitly requested right alignment.
