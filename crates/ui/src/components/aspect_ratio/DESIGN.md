# AspectRatio design

`AspectRatio` reserves a full available-width rectangle and clips a caller-owned child UI to that rectangle. It paints no background and adds no semantic role of its own; content and accessibility semantics come from the closure.

`show` reads the available width, then asks the handler to sanitize the configured width/height ratio and calculate the height. It allocates a hover-sense response and builds a child `Ui` with the same rectangle and clip. The closure's return value is passed through unchanged alongside the allocation response.

Non-finite, non-positive, or `<= 0.001` ratios use a square fallback. Height is `width / ratio`; the caller's parent layout controls the width. The allocation is one rectangle plus one child UI per call, with no retained state or animation. Very narrow parents produce correspondingly narrow regions.
