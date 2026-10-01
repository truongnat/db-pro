# Overlay design

`mod.rs` is the public facade for floating surfaces, popovers, dropdowns, context menus, tooltips, and toast notifications. `ui.rs` owns egui layout and painting, `tooltip.rs` positions hover help, `toast.rs` owns toast presentation and timed queue state, `handler.rs` contains pure outside-click policy, and `config.rs` contains overlay-specific measurements.

## Event flow

Popover and dropdown builders borrow an `open` flag and a trigger `Response`. A trigger click toggles the flag; when visible, the component draws a foreground `Area`, then checks pointer interaction against the popup and trigger rectangles. Clicks outside both close the popup. Dropdown selection returns the selected item index and closes the menu; disabled rows cannot be selected. Context-menu helpers use the triggering response and egui pointer context to render caller-provided menu items.

Tooltip reads the trigger response's hover state, animates visibility, chooses the requested side with a one-side flip when it would cross the screen edge, then clamps its position into the viewport. Toast can be rendered as one builder or managed as a timed collection. The manager advances elapsed time from egui's stable frame delta, removes expired items, stacks by position, and returns `(toast_id, ToastResponse)` pairs for actions and close buttons.

## Rendering cost and layout limits

Each visible popup or tooltip draws one foreground area and its content. Closed popovers/dropdowns and hidden tooltips skip their areas. The toast manager visits each live toast each render and groups them by the six supported screen positions; memory and work grow with the number of active toasts. Toast text and menu contents are caller-provided, so long content can increase surface size. Keep menu labels concise and ensure the parent viewport has enough room for the desired interaction.

## Accessibility and motion

Use the trigger's accessible label and keyboard interaction in the caller; this module does not assign application-specific names to popovers or menus. Dropdown rows honor the enabled flag, visually expose selection/danger state, and return an index so callers can apply the action. Toast action and close controls return explicit click results. Tooltips are non-interactive and should supplement, not replace, visible labels. Overlay transitions use the shared animation helpers; tooltip's hover fade is short and decorative. Keyboard dismissal/focus management for popovers and dropdowns remains the caller's responsibility; these surfaces currently close on outside click, not Escape.
