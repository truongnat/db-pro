# Issue #502 — Verification

- `cargo check -p db-pro-core` — clean.
- `cargo test -p db-pro-core` — 502 passed, 0 failed, including:
  - `literal_null_text_is_masked_like_data`: `Some("null")`/`Some("NULL")` →
    `"***"` under Redact; `Some("null")` → `"null"` verbatim under
    PreserveNull.
  - `real_null_is_preserved_for_every_rule`: `mask_cell(None, …)` → `"NULL"`
    for all six rules.
  - `row_passes_real_null_through`: `None` cell → `"NULL"` in `mask_row`.
- `cargo test -p db-pro-app masking` — 2 passed (`build_preview` call sites
  unchanged; `String` cells wrapped internally).
- `cargo check --workspace` — clean.
