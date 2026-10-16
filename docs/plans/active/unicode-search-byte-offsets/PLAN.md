# Unicode Search Byte Offsets

State: REVIEW.
Issues: truongnat/db-pro#445 and #509.
Branch: `fix/unicode-search-byte-offsets`.

## Goal

Keep case-insensitive search and highlight ranges valid in the original UTF-8 buffer when Unicode lowercase expansion changes byte length.

## Evidence

Both query-document search and SQL-editor highlight search lowercased the entire text, then reused byte offsets from that transformed string against the original text. Characters such as `İ` lowercase to multiple Unicode scalars and a different byte length.

## Scope

- Add a shared Unicode-insensitive matcher that maps folded scalar spans to original byte ranges.
- Use it for query-document matches and SQL-editor highlight ranges.
- Test expanding lowercase mappings and ASCII behavior.

## Non-goals

- No changes to locale-specific collation or full Unicode case folding beyond Rust's existing lowercase behavior.
- No visual redesign or search navigation behavior changes.

## Acceptance

- [x] Matches return valid byte ranges in original source text.
- [x] Both query-document navigation state and SQL-editor highlight use the shared ranges.
- [x] Focused tests, UI/app/native check, formatting, and clean-code scan pass.
