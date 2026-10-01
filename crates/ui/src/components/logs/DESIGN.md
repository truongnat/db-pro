# Logs design

`LogViewer` receives a borrowed slice of entries and paints it in the given order. The handler maps each level to a semantic icon/color and chooses message color; the UI paints timestamp, icon, and message in one horizontal row. Empty input renders a muted empty message. The viewer owns no filtering, ordering, storage, or pagination state.

Each entry exposes its timestamp and message as normal egui labels. Level meaning is represented by both icon and text color, so color is not the only signal. Long, unbroken messages can exceed the available row width; place the viewer inside an appropriate scrolling/width-constrained parent when needed.

Rendering is O(n) in entry count with three text elements per row and no retained cache. Shared font, stroke, radius, spacing, and icon sizes are read directly from `tokens.rs`; only the component-owned empty message remains in `config.rs`.
