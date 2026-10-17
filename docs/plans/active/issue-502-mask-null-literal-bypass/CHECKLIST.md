# Issue #502 — Checklist

- [x] `mask_cell(Option<&str>)`; `None` → `"NULL"` for all rules
- [x] Removed `eq_ignore_ascii_case("null")` bypass
- [x] `mask_row` accepts `&[Option<&str>]`
- [x] Updated `preview_masking` and `mask_transfer_batch` call sites
- [x] Updated app callers (`build_preview`, `masked_csv_export_outcome`) — no signature change needed
- [x] Tests: literal "null" masked; `None` preserved for every rule
- [x] `cargo test -p db-pro-core masking` — 9 passed
