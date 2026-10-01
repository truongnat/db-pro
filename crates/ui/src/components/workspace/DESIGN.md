# Workspace design

`mod.rs` preserves the public API. `ui.rs` paints the activity rail, status bar, and connection indicator. `handler.rs` holds display state types and the pure layout/threshold decisions used by those widgets. `config.rs` contains workspace-specific measurements and the static activity destination definitions.

## Event and rendering flow

The caller supplies status items, selected activity destination, connection health, and optional latency on each frame. `ActivityBar::show` returns a selected destination intent; it does not change the workspace tab. `StatusBar::show` paints left/right items and exposes each item label and optional hover tooltip. `ConnectionIndicator::show` paints caller-provided connection state and accessible text; it does not ping or connect.

Status-bar items are measured from their full label and placed from the left/right edges. The activity bar paints four entries in canonical order: Explorer, Query Editor, Agent, Diagram. The enum retains Settings for compatibility, but no Settings item is rendered by this component. Layout is constant with respect to caller data size except status text measurement.

## Accessibility and viewport limits

Activity controls expose button labels, keyboard focus, and the shared theme focus outline. Status items expose label/tooltip text; the connection indicator exposes name, driver, health, and optional latency as a label. A latency over 200 ms uses warning color. Left and right status item groups are not collision-managed; callers should keep labels concise or shorten them in narrow windows. All color decisions use `DbProTheme` semantic tokens.
