# Tabs design

`mod.rs` is the stable public facade. `ui/` owns segmented and underline egui layout, hit collection, widget metadata, focus rings, and indicator painting. `layout.rs` collects per-frame tab geometry and assigns stable track IDs. `handler.rs` converts click and keyboard signals into a selected index. `track.rs` computes the moving indicator. `style.rs` maps active/hover state through the theme; `config.rs` owns track dimensions and timing.

Each visible tab is measured and painted once per frame. The UI records hit rectangles and focus, then sends the click/focus/key signals to the handler. The selected index feeds the active indicator. Reduced motion skips indicator interpolation and repaint scheduling. The cost is linear in the number of labels, with one layout measurement and hit target per label.

Arrow keys move between tabs with wraparound; Enter/Space activates the focused tab. Tab targets publish radio-style widget metadata and request focus after pointer activation. `SegmentedTabs::focusable(false)` disables focus and keyboard participation for that style. The selected index is caller-owned and must stay within the label slice to represent a selected tab; an empty list draws no indicator.

Long labels contribute their full measured width. Place the track in a horizontal scroll area when labels can exceed the viewport. Keep labels short, ensure theme text and accent colors remain legible in light and dark modes, and avoid using tabs for an unbounded list.
