# Issue #502 — Literal "null" bypasses masking

## Root cause
`mask_cell(value: &str, …)` treated the *text* "null" (any ASCII case) as SQL
NULL and returned `"NULL"` before applying the column rule. A `&str` cannot
distinguish a real SQL NULL from the literal string "null", so:

- a cell containing the text "null" skipped masking entirely, and
- the emitted "NULL" was indistinguishable from a real NULL in the export.

## Fix
- `mask_cell(value: Option<&str>, rule, keyed, key_material) -> String`
- `None` (SQL NULL) is preserved verbatim as `"NULL"` for *every* rule —
  masking must never rewrite a real NULL into masked-looking content.
- The `eq_ignore_ascii_case("null")` early return and the dead `PreserveNull`
  empty-check were removed; `Some("null")` is ordinary data and gets masked.
- `mask_row` now takes `cells: &[Option<&str>]` so NULL-typed inputs can reach
  `mask_cell`; unmatched `None` cells emit `"NULL"`.
- `preview_masking` / `mask_transfer_batch` wrap their `String` cells in `Some`.

## Out of scope
`TransferRow.cells: Vec<String>` still cannot represent a real SQL NULL — the
transfer path stringifies values upstream. Changing that type would touch the
transfer pipeline; the masking API is now NULL-capable for callers that have
typed cells (e.g. `UiCell::Null → None`).
