# Fix handoff — slice 02 query editor/workspace (QueryEditorFixes)

Implemented the two actionable findings from `02-query-editor-workspace.md`.

## 1. P1 — UTF-8 panic in `extract_word_prefix`

`crates/ui/src/query/schema_completion.rs`

- `extract_word_prefix` now walks `char_indices().rev()` instead of
  `text[end-1..end]` byte-decrement slicing, which panicked when the cursor sat
  directly behind a multi-byte char (e.g. `SELECT tên`, `SELECT 🚀`).
- Semantics preserved (Unicode `is_alphanumeric`/`_` tail; empty prefix on
  trailing non-word char or empty input).
- Regression test `test_extract_word_prefix_multibyte_tail_does_not_panic`
  covers Vietnamese `tên`/`bảng`, emoji, and the ASCII baseline.

## 2. Acceptance gap — confirmed-hover occurrence highlights

`crates/ui/src/editor/renderer.rs`, `crates/ui/src/query_editor_surface_view.rs`,
`crates/ui/src/editor/renderer_tests.rs`

- `SqlEditor::rich_hover_open: bool` → `rich_hover: Option<HoveredSqlToken>`
  (additive field swap; the confirmed token is already produced by
  `HoverState` in `QueryDocument`). Builder renamed
  `with_rich_hover_open` → `with_rich_hover`; diagnostic-tooltip guard uses
  `self.rich_hover.is_none()` (unchanged behaviour).
- New `paint_hover_occurrences` pass in `paint_highlights` (after
  search/execution, before selection): washes every *other* token spelling the
  hovered word with `theme.accent.linear_multiply(0.22)` — theme token only.
  Scoped to the visible viewport window (+2-line overscan, same formula as
  `paint_visible_lines`) so the scan stays cheap on large buffers.
- Matching uses a pure helper `hover_occurrence_ranges(text, anchor, tokens,
  window)` with Unicode case-insensitive comparison (`str_eq_ignore_case` —
  `USERS`/`users`/`bảng` match; `USERS_archive` correctly does not).
- Surface passes `doc.hover_state.confirmed_token()` straight through — only
  the 500 ms-confirmed hover path highlights; raw/pending hovers do not.
- Unit tests: case-insensitive match + anchor skip, visible-window clipping,
  out-of-range anchor safety, Unicode case equality.

## Files changed

- `crates/ui/src/query/schema_completion.rs`
- `crates/ui/src/editor/renderer.rs`
- `crates/ui/src/query_editor_surface_view.rs`
- `crates/ui/src/editor/renderer_tests.rs`

## Verification

- `cargo fmt --check -p db-pro-ui` — clean.
- `cargo check -p db-pro-ui` — clean (final run after sibling mid-flight
  edits in `tokens/semantic.rs` / `object_mutation_service.rs` /
  `schema_workbench*` settled).
- `cargo test -p db-pro-ui --lib -- extract_word_prefix hover_occurrences
  str_eq_ignore_case` — 5/5 passed.
- `cargo test -p db-pro-ui --lib -- schema_completion::tests
  editor::renderer::tests` — 34/34 passed (full touched-module suites).

## Left

- P2s from the slice report intentionally out of scope: `UiQueryResult` clone
  per frame (`query_view.rs:270`), sync `SchemaSymbolIndex::build`
  (`schema_events.rs:60`), unbounded unscoped completion scan, manual-paint
  context chip.
