# Findings

## P1 — Editor content size unused; no scroll

`content_width` / `content_height` are computed then discarded (`let _ = ...`).
Long queries cannot be scrolled with the wheel — only clipped.

## P2 — Caret never blinks; chrome feels heavy

Solid 2px caret always on; focus ring stroke is loud; line height floors at 22px without Zed-like leading.
