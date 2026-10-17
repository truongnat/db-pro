# #509 — Search highlight offsets corrupt on Unicode case expansion

## Goal
Case-insensitive search must produce `(start, end)` byte ranges in the
ORIGINAL text — never offsets into a lowercased copy.

## Root cause
`EditorSearchState::update_matches` (and a duplicated loop in
`editor/renderer.rs:444-481`) ran `match_indices` on `text.to_lowercase()`.
Case folding changes byte width (`İ` U+0130 → "i\u{307}" 2→4 bytes; Kelvin
`K` U+212A → "k" 3→1 byte), so every range after such a char shifted off the
original text → wrong highlighting / mid-char slice panic.

## Fix
- New `crates/ui/src/editor/search.rs`:
  `case_insensitive_matches(text, query)` iterates `char_indices` on the
  original text and compares per-char `to_lowercase()` iterators, yielding
  non-overlapping `(start, end)` ranges on original char boundaries.
  Documented contract: expansion chars like `İ` compare as their expansion
  and do NOT equal a single query char — boring, predictable find behavior.
- `query/query_document.rs::update_matches` and
  `editor/renderer.rs::paint_search_and_execution` now share the helper —
  search state and painted highlights can no longer diverge.

## Tests
`editor/tests/search_tests.rs` (ascii, empty/overlong queries,
non-overlap, `İ`-expansion offset shift, expansion-char no-match, Kelvin
`K`, char-boundary ends) +
`query/tests/query_document_tests.rs::search_match_ranges_are_original_
text_byte_offsets`.
