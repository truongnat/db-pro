# Native Core UI Modernization — Findings

## F1 — Tooltips are inline-only; no rich tooltip primitive exists

**Evidence**: `crates/ui/src/components.rs` defines `icon_text(...)` and the
egui-native `.on_hover_text(...)` is used in 30+ call sites
(see `grep -rn on_hover_text crates/ui/src`).

**Failure scenario**: A modern IDE/DB tool surfaces a label plus its keyboard
shortcut in the tooltip. Without that, users have to memorize or look up the
shortcut in a help dialog. Linear, Vercel, Raycast, Cursor all show the
shortcut inline in the tooltip.

**Severity**: P2.

**Smallest coherent fix**: Add a `tooltip(ui, body, text, shortcut, theme)`
wrapper that:
1. shows `on_hover_text(text)` on the wrapped response;
2. when hovered, also paints a `kbd_chip` next to the text inside a tooltip
   popup using egui's `show_tooltip_at` API.

## F2 — Spinner / skeleton are missing; loading states are jarring

**Evidence**: `crates/ui/src/components.rs` has no `spinner` or `skeleton`
function. `egui::Spinner` is used in some places (e.g. `runtime progress`)
but the default egui spinner is visually flat.

**Failure scenario**: When the agent provider is loading or the schema is
refreshing, the UI shows a static text label. Modern apps use either a small
spinner next to the action or a skeleton placeholder in the loading region.

**Severity**: P2.

**Smallest coherent fix**: Add `spinner(ui, color, theme)` that paints a
rotating arc using `ctx.animate_value` (rotation 0..1 → 0..360°).
Add `skeleton(ui, width, height, theme)` that paints a rounded rect filled
with `surface_hover` and overlays a moving shimmer using
`ctx.animate_value` (offset 0..1 → -width..+width).

## F3 — Switch / segmented control primitives are missing

**Evidence**: `crates/ui/src/components.rs` does not define `switch` or
`segmented_control`. The connection dialog and settings use raw egui
`ComboBox` or text labels.

**Failure scenario**: Boolean settings (readonly, SSL mode) and mode pickers
(driver, output tab) look like raw form fields rather than the modern
toggle/picker pattern.

**Severity**: P2.

**Smallest coherent fix**: Add `switch(ui, &mut bool, label, theme)` that
paints a track + thumb with smooth thumb animation via
`ctx.animate_value_bool`. Add `segmented_control(ui, &[&str], selected,
theme) -> Option<usize>` that paints a pill bar with the selected segment
highlighted using `surface_active`.

## F4 — No kbd chip for keyboard shortcuts

**Evidence**: `crates/ui/src/palette_view.rs:18` stores `shortcut:
Option<String>` and renders it as a `RichText` label. There is no styling
that visually distinguishes a shortcut from regular text.

**Failure scenario**: Keyboard shortcuts are listed but don't pop visually.
Modern apps use rounded monospace pills (Linear's `⌘K`, Vercel's `⌘P`).

**Severity**: P2.

**Smallest coherent fix**: Add `kbd_chip(ui, label, theme)` that draws a
rounded monospace pill with `surface_elevated` fill and `border_default`
stroke. Use it in `palette_view.rs` and on topbar buttons that have
shortcuts.

## F5 — Status indicator is a glyph, not a glowing dot

**Evidence**: `crates/ui/src/app.rs:579` `connection_indicator` returns
`Icon::CircleCheck` / `Icon::Circle`. `crates/ui/src/navigation_view.rs:32`
uses `icon_text(connection_icon, "", connection_color)`.

**Failure scenario**: Trending apps (Linear, Raycast) use a small colored dot
that can pulse for live state. The current glyph-based indicator looks
utilitarian.

**Severity**: P2.

**Smallest coherent fix**: Add `status_dot(ui, color, pulsing, theme)` that
paints a filled circle with optional outer ring pulse (multi-layer alpha,
pulse radius animated via `ctx.animate_value_bool`).

## F6 — No toast / snackbar for transient notifications

**Evidence**: `crates/ui/src/app.rs:304` stores `runtime_message: String`
that is shown in the statusbar only. There is no toast/snackbar primitive.

**Failure scenario**: When the user saves a query, the feedback is "Saved
query" appearing in the statusbar. Modern apps use a transient toast
that fades in/out near the cursor or in the bottom-right corner.

**Severity**: P2.

**Smallest coherent fix**: Add `toast(ui, level, message, theme)` that
paints a small rounded panel with a colored left accent strip and a
subtle elevation shadow. Use it for `runtime_message` feedback when the
message is at "Success" or "Failed" level.

## F7 — Search input has no leading icon, no clear button

**Evidence**: `crates/ui/src/explorer_view.rs` and `result_grid_view.rs` use
the basic `input(ui, &mut value, hint, width, theme)` helper. The user has
no visual hint that the field is a search field, and no quick way to clear.

**Severity**: P2.

**Smallest coherent fix**: Add `search_input(ui, value, hint, leading_icon,
width, theme)` that:
1. allocates an inner rect inside the input rect for a leading icon and a
   clear X;
2. renders the leading icon in `text_muted` color;
3. shows the clear X only when `value.is_empty() == false`;
4. adjusts the input text padding to leave room for both icons.

The existing `input` API stays untouched.

## F8 — Empty state lacks an action slot

**Evidence**: `crates/ui/src/components.rs:141` `empty_state(ui, icon, title,
description, theme)` does not take a closure for a CTA button.

**Failure scenario**: When no connection is selected, the empty state could
suggest "Add a connection" with a button, but currently shows text only.
Modern apps always offer a next action from the empty state.

**Severity**: P2.

**Smallest coherent fix**: Add `empty_state_with_action(ui, icon, title,
description, theme, action: impl FnOnce(&mut Ui) + 'static)` and keep the
existing `empty_state` as a thin wrapper that passes `|_| {}` to the new
function.

## F9 — Custom-painted micro-polish opportunities

**Evidence**: `crates/ui/src/workspace_view.rs` already calls
`paint_tab_indicator(...)` (a 2px left accent line). Modern apps animate
the indicator sliding between tabs.

**Severity**: P3 (visual polish, not a correctness defect).

**Smallest coherent fix**: Replace the static 2px left line with a 2px tall
underline that interpolates between the previous and current tab's center-x
using `ctx.animate_value_with_time`. Skip the animation when
`reduce_motion` is true.
