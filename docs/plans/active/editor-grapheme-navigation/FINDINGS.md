# Findings

## Confirmed P2 — editor operations split Unicode grapheme clusters

- Evidence: cursor and renderer deletion stepped across UTF-8 scalar boundaries; word selection classified combining marks as non-word scalars.
- Impact: caret movement, backspace/delete, and double-click selection could detach accents or split ZWJ emoji.
- Scope: retain byte offsets while calculating user-facing movement/selection by extended grapheme boundaries.
- Confidence: high; reproduced by direct boundary cases in `e + U+0301`, Vietnamese multi-mark text, and `👩‍💻`.
