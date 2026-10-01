# Responsive layout design

`Container` measures local available width, resolves a bounded content width and gutter, then renders the caller closure in a centered child region. `ResponsiveGrid` calculates a column count from available width, minimum cell width, gap, and maximum columns; it allocates row-wise cells and invokes the caller once per item. Pure width and grid decisions are kept separate from egui allocation.

The layout responds to the immediate parent width, so it works in split panes and narrow panels as well as full windows. It does not own scrolling or child overflow policy; each cell is width-bounded, while its contents remain caller-controlled.

Container layout is constant work. Grid layout calculates metrics once and performs O(n) item allocation/closure calls with O(1) layout state. Avoid using it for enormous data sets that require virtualization.
