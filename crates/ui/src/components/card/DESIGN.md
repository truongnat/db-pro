# Card design

`Card` is a passive themed frame. Its `show` method delegates child layout to egui's frame and returns the closure result. `card_header`, `card_content`, and `card_footer` are composition helpers; the footer divider is decorative. None adds click or focus behavior.

`MetricCard` lays out a title, primary value, optional icon, and optional trend. The handler resolves trend direction independently from semantic tone and chooses the matching icon/color; egui measures and paints the resulting text. Trend text is exposed as a label. Shared surface, spacing, radius, and padding use the theme and shared tokens, while metric typography/density remain local.

Cost grows with the content built by the caller; the card adds a frame and a fixed number of child widgets. It does not constrain or truncate long content. Responsive grids and overflow handling belong to the parent.
