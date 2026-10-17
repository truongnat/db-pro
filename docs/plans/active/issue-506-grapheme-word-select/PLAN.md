# #506 — select_word_at splits words at combining marks

## Goal
Double-click word selection must treat grapheme clusters as indivisible so
combining marks (U+0301, Vietnamese combining tones U+0302+U+0301) no longer
split words: `café` (decomposed) selects the whole word.

## Root cause
`select_word_at` classed each **char** with `is_alphanumeric() || '_'`.
Combining marks are non-alphanumeric → clicking/expanding across them broke
the run, and clicking the mark itself selected just the mark.

## Fix
`crates/ui/src/editor/selection.rs` — expand by grapheme cluster
(`unicode-segmentation::grapheme_indices(true)`, new workspace dep): the
clicked cluster is classified by its base char (`alphanumeric || '_'`);
expansion walks cluster boundaries and breaks on whitespace or a class
change — identical semantics to the old classifier, with cluster atomicity
added. Offset at end-of-buffer selects the last cluster (old `prev_char`
behavior).

## Dependency
`unicode-segmentation = "1"` added to `[workspace.dependencies]` and
`db-pro-ui` (justified per issue; already in the lock file transitively).

## Tests
`editor/tests/selection_tests.rs`: decomposed `café` (click on mark and on
base char), decomposed Vietnamese `ấ`, end-of-buffer last-word selection.
