# Separator design

Separator allocates hover-sense geometry and paints a themed line. Horizontal separators span the available width and may include a centered label with a line segment on each side. Vertical separators fit the current row height up to a component-defined cap. Handlers calculate allocation sizes and line/label positions; egui provides text measurement and clipping.

A labeled horizontal divider exposes its text as label metadata. Unlabeled horizontal and vertical lines are decorative because egui 0.29 has no separator role. None is focusable or interactive. The line uses the theme's subtle border color, and shared spacing comes directly from `tokens.rs`.

Each separator paints a constant number of line segments and, when labeled, measures one label. The horizontal line is clipped to its allocated rectangle, so long labels do not paint into adjacent content; a narrow parent can leave little or no visible line around the label.
