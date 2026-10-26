#!/usr/bin/env python3
"""Flag captures that show egui's widget-id clash overlay.

When two widgets claim the same egui `Id` — the usual suspects are sibling
`ScrollArea`s without `id_salt` — egui paints a 1px border plus a
"🔥 Nth use of ScrollArea ID …" label in `Visuals::error_fg_color`
(egui-0.36.2 `Context::check_for_id_clash`). This app maps that token to its red
status colour, so the overlay is a band of red-on-grey text over the UI.

The shipped binary turns `warn_on_id_clash` off in release builds, so this scan
only finds anything on debug captures — which is where a clash must be fixed,
because a shared id also means two areas silently share scroll position.

A real error banner uses the same colour, so a hit is a candidate to eyeball,
not a verdict.

Usage:
  python3 id_clash_scan.py <png> [...]
  python3 id_clash_scan.py --dir <dir>          # recursive *.png
Exit code 1 when a capture is flagged.
"""

import glob
import os
import sys
from collections import Counter

from PIL import Image

# Red status colour (red-500/red-600 family): strong red, no blue cast.
MIN_RED = 190
MAX_GREEN_BLUE = 120
MAX_CHANNEL_SPREAD = 25
# The overlay labels are ~250 px of glyph coverage each; a stray antialiased
# border pixel should never reach this.
MIN_FLAGGED_PIXELS = 200


def clash_pixels(path: str) -> int:
    """Pixels that look like the id-clash overlay's error colour."""
    counts = Counter(Image.open(path).convert("RGB").getdata())
    return sum(
        n
        for (r, g, b), n in counts.items()
        if r >= MIN_RED and g <= MAX_GREEN_BLUE and b <= MAX_GREEN_BLUE and abs(g - b) <= MAX_CHANNEL_SPREAD
    )


def main(argv: list[str]) -> int:
    paths = (
        sorted(glob.glob(os.path.join(argv[1], "**", "*.png"), recursive=True))
        if argv and argv[0] == "--dir"
        else argv
    )
    if not paths:
        print("give PNG paths or --dir <capture-dir>", file=sys.stderr)
        return 2

    flagged = []
    for path in paths:
        count = clash_pixels(path)
        hit = count >= MIN_FLAGGED_PIXELS
        flagged.append(path) if hit else None
        print(f"{'CLASH' if hit else 'clean':6} {count:6} px  {os.path.basename(path)}")

    print(f"\n{len(paths)} captures scanned, {len(flagged)} flagged for the id-clash overlay")
    return 1 if flagged else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
