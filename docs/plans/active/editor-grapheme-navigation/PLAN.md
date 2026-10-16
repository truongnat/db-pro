# Editor Grapheme Navigation

State: REVIEW.
Issues: truongnat/db-pro#506 and #472.
Branch: `fix/editor-grapheme-navigation`.

## Goal

Keep cursor movement, deletion, and double-click word selection aligned to extended grapheme boundaries while retaining UTF-8 byte offsets in the buffer.

## Evidence and failure

`CursorPosition` and editor Backspace/Delete used scalar boundaries, and `select_word_at` classified every scalar independently. Combining marks were therefore detached from their base letters; ZWJ emoji sequences could be split by movement or deletion.

## Scope

- Add `TextBuffer` grapheme-boundary helpers backed by `unicode-segmentation`.
- Use grapheme boundaries for cursor left/right, word movement, Backspace/Delete, and auto-pair skip.
- Select contiguous grapheme runs so combining marks stay with alphanumeric bases.
- Add cursor, selection, and renderer key-event regression tests.

## Non-goals

- No change to byte-offset storage, syntax token offsets, SQL parsing, or multi-line layout.
- No visual redesign or provider behavior change.

## Acceptance

- [x] Cursor and deletion operate on complete grapheme clusters (combining sequences and ZWJ emoji).
- [x] Double-click selection includes combining marks and keeps existing ASCII identifier behavior.
- [x] Focused editor tests, formatting, and app/native check pass.
