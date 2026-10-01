# Workspace API

`ActivityBar`, `ActivityBarItemKind`, `StatusBar`, `StatusBarItem`, `ConnectionIndicator`, and `ConnectionHealth` are re-exported through `components::workspace` and `components`.

## ActivityBar

`ActivityBar::new(selected, theme).show(ui) -> Option<ActivityBarItemKind>` returns the clicked destination. Rendered destinations are Explorer, QueryEditor, Agent, and Diagram. `Settings` remains a public enum value but has no rail button here. The caller updates navigation state.

## StatusBar

`StatusBarItem::new(text)` creates an informational item; chain `icon(Icon)`, `tooltip(text)`, and `accent(bool)`. Its fields are public. `StatusBar::new(left_items, right_items, theme).show(ui) -> Response` paints items from both edges and returns the overall egui response. Each item exposes its label and, when configured, a hover tooltip. The component does not resolve overlap if both groups exceed available width.

## ConnectionIndicator

`ConnectionHealth` is `Healthy`, `Degraded`, or `Disconnected`. `ConnectionIndicator::new(name, driver, health, theme).latency(milliseconds).show(ui) -> Response` displays supplied state. Latency strictly greater than 200 ms gets warning color. The returned response is a labeled status surface; the component does not measure live latency or alter connection state.
