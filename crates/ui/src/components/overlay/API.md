# Overlay API

The public facade is `components::overlay`; the same public types and helpers are re-exported through `components`.

## Tooltip

`Tooltip::new(text, theme).position(TooltipPosition::{Top, Bottom, Left, Right}).shortcut("Ctrl+C").show(&response)` draws non-interactive hover help and returns a clone of the trigger `Response`. Top is the default. Placement flips once toward the opposite side at an edge and clamps to the viewport. The tooltip is hover-only, has no caller-owned open state, and is not a substitute for a persistent label.

## Popover and dropdown

- `Popover::new(&mut open, theme).show(ui, &trigger_response, closure)` toggles on trigger click and returns `Option<R>` from its content closure while visible. A click outside both popup and trigger closes it.
- `DropdownItem::new(label)` supports `icon(Icon)`, `shortcut(text)`, `enabled(bool)`, `selected(bool)`, and `danger(bool)` builder methods. Fields are public for direct construction.
- `DropdownMenu::new(&mut open, items, theme).show(ui, &trigger_response)` returns `None` while closed and `Some(index)` while visible; a selected enabled row returns its zero-based item index and closes the menu. Disabled rows do not return an index. The component does not execute the selected action.

## Context menu and surface helpers

`is_context_menu_triggered(&response, ui)` checks whether the response activated the context-menu gesture. It supports secondary click, macOS Ctrl/Cmd click, and Shift+F10 on a focused widget. `context_action_menu(ui, &response, theme, closure)` renders caller-supplied actions; the closure receives `(&mut Ui, &mut bool)` so an action can request dismissal. Escape and outside clicks also close this menu. `ctx_menu_item(ui, icon, label, shortcut, color, theme)` renders a menu row and returns its `Response`. `floating_surface(theme, rounding, margin)` returns an egui `Frame` styled with the theme's floating surface tokens.

## Toast

`Toast::new(message, theme)` defaults to `ToastVariant::Default`, bottom-right placement, and a close button. Configure with `variant(Default|Success|Danger)`, `position(ToastPosition)`, positional shortcuts (`top_left`, `top_center`, `top_right`, `bottom_left`, `bottom_center`, `bottom_right`), `action(label)`, and `closable(bool)`. `show(ui)` and `show_floating(ui, salt)` return `ToastResponse { action_clicked, dismiss_clicked }`; application state changes and actual dismissal are caller-owned.

`ToastManager` stores queued `ToastItem`s. `show(message, variant, position)`, `show_with_action(...)`, and `success/error/info(message, position)` return monotonically increasing `u64` IDs. Plain items expire after five seconds; action items after eight seconds. Use `dismiss(id)`, `clear()`, `is_empty()`, or `len()` to manage the queue. `render_ctx(ctx, theme)` / `render(ui, theme)` advances timers and returns `(id, ToastResponse)` for interactions; expired items are removed before rendering. These timers use egui frame delta, so they pause when rendering pauses.

## Interaction boundaries

Popover and dropdown dismissal is pointer-outside based. They do not provide Escape handling, keyboard roving, or focus restoration. Toasts do not expose live-region semantics through this API; callers should announce important outcomes through the application's accessible status channel as well. Constrain very long labels and messages in the caller because popup and toast content is not truncated by the API contract.
