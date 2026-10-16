# Findings

## Confirmed P2 — lowercased byte offsets do not address original text

- Issues: #445 and #509.
- `İ` (two UTF-8 bytes) lowercases to `i` plus COMBINING DOT ABOVE (three bytes). Searching a later character in the lowercased copy yields an offset shifted from the original source.
- Query-document matches and editor highlights both used those transformed offsets against original text, risking misplaced rendering and invalid UTF-8 boundaries.
- Fix: map lowercased scalar start/end positions back to each original scalar's byte span.
