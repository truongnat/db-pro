# #509 Findings

## Evidence
- `query/query_document.rs:61-65` (`update_matches`) and the duplicated
  paint loop `editor/renderer.rs:444-447` both ran
  `text.to_lowercase().match_indices(&query.to_lowercase())` and then used
  those indices on the original text.
- Unicode full case folding changes byte width per char:
  `İ`(U+0130,2B)→`i\u{307}`(4B), Kelvin `K`(U+212A,3B)→`k`(1B). Every match
  after such a char is shifted → wrong highlight; a shifted end can land
  mid-char → `&text[s..e]` panics (renderer `range_to_screen_rects` slices).
- The two sites had already diverged in behavior risk; now share
  `editor::search::case_insensitive_matches`.

## Decision
Per-char lowercase comparison (`char::to_lowercase` iterators) keeps
original byte offsets and non-overlap semantics of `match_indices`.
Documented limitation: expansion chars compare as their expansion, so `İ`
never equals query 'i' — matches the issue's "boring correct behavior" note;
full case-folded position mapping is not needed for a find UI.

## Severity
P2 — panic/mis-highlight whenever the buffer contains expansion-fold chars.
