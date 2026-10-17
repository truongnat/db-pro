# Issue #502 — Findings

- `mask_cell` had *three* NULL-related shortcuts: a top-level
  `eq_ignore_ascii_case("null")` bail-out for all rules, a redundant
  `PreserveNull` empty-string check, and a nested `PreserveNull` "null" check.
- All production callers feed `Vec<String>` cells (`preview_masking` sample
  rows in `app::tools::masking::build_preview`, `mask_transfer_batch` over
  `TransferRow.cells` in `transfer_harness_view.rs`). No caller held `UiCell`,
  so `mask_row` accepts `&[Option<&str>]` to carry NULL-ness without churning
  `TransferRow`.
- `PreserveNull` on non-null values already returned the value verbatim, so
  removing the special-cases leaves that rule's semantics unchanged.
