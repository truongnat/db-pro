# Dialog design

`mod.rs` is the public facade. It exposes the layered implementation and preserves `dialog::modal::{Dialog, DialogActionLabels, close_icon_button, dialog_actions}` for existing callers. `Dialog` owns no domain state: it borrows the caller's `open` flag and returns the value produced by the content closure while the overlay is visible.

## Layers

- `ui.rs` lays out the dialog, paints the dim layer and card, runs the content closure, and applies the open/close motion.
- `frame.rs` separates the scrollable body from the optional footer so callers can keep actions visible while content scrolls.
- `sheet.rs` paints the right-side panel and its close control using the same modal coordination.
- `layout.rs` contains shared egui geometry and paint operations for dialog and sheet.
- `handler.rs` resolves Escape and backdrop signals into a dismissal reason. The caller-owned flag is changed only by the component's dismissal policy; content actions remain caller decisions.
- `modal_guard.rs` records the modal render stack and keeps keyboard focus in the topmost modal layer.
- `config.rs` holds dialog and sheet dimensions used by these surfaces.

## Event flow and rendering cost

The caller draws the builder each frame. A closed overlay returns `None`; an opening or open overlay registers its stable egui ID, resolves topmost ownership, handles Escape/backdrop input, draws one dim layer and one foreground area, then invokes the content closure. Escape takes precedence over a simultaneous backdrop click. A backdrop click inside the card is ignored. Sheets do not dismiss on backdrop clicks. Only the topmost registered modal owns modal input and focus trapping.

While the transition is active, the overlay requests repaint so the animation can finish. Once closed motion reaches zero, no overlay area or content closure is drawn. Cost is proportional to the content the caller draws; the modal stack check is linear in the small number of simultaneously open modals.

## Focus and sizing

A hidden focus anchor provides an in-layer fallback if egui focus escapes to the underlying window. Content may request focus after that fallback is applied. The component does not move focus back to the original trigger when it closes; callers that need that behavior should retain the trigger response and request focus after closing.

Dialog width defaults to the component's configured width and is clamped internally to a minimum content width. `DialogFrame::body` is scrollable within the available height; `footer` stays outside that scroll region. On narrow windows, callers should keep body content responsive and avoid assuming the requested width fits the screen. `Sheet` similarly accepts a width but its content remains the caller's layout responsibility.

## Accessibility and motion

Use a descriptive title and optional description; interactive content should expose meaningful labels and remain keyboard-operable. Close controls use the shared button implementation. Dialog and sheet transitions use the shared overlay animation helper, which honors the theme's reduced-motion setting. The current focus trap is a layer escape guard rather than a full Tab/Shift+Tab cycle among descendants; keyboard traversal details depend on egui's focusable widgets.
