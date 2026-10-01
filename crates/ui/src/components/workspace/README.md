# Workspace components

Native egui components for the activity rail, status bar, and current connection summary. They present state supplied by the caller and return navigation intent; they do not own application or connection state.

`workspace` keeps its stable public API while separating responsibilities:

- `mod.rs` — module boundary and public re-exports.
- `ui.rs` — egui painting and widget builders for `StatusBar`, `ActivityBar`, and `ConnectionIndicator`.
- `handler.rs` — public state types plus label, geometry, and threshold decisions.
- `config.rs` — workspace-specific dimensions and activity destination metadata.

Activity item order is Explorer, Query Editor, Agent, and Diagram. Status items are caller-provided and painted from both edges. Connection latency above 200 ms uses warning color. Long status items can overlap on narrow layouts, so callers should keep them concise. Activity controls and status entries expose accessible labels/tooltips; focused activity items have a theme focus outline.

```rust
let left = [StatusBarItem::new("public.users").icon(Icon::Table)];
let right = [StatusBarItem::new("Connected").accent(true)];
StatusBar::new(&left, &right, theme).show(ui);

if let Some(destination) = ActivityBar::new(ActivityBarItemKind::Explorer, theme).show(ui) {
    navigate_to(destination);
}
```
