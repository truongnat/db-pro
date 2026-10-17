# Issue #503 — Checklist

- [x] `mask_row` rule lookup uses `eq_ignore_ascii_case`
- [x] `unmatched_mask_rules(headers, profile)` helper
- [x] `MaskingPreview.unmatched_rules` + message suffix surfaced in preview
- [x] `tracing::warn!` on unmatched rules in `mask_transfer_batch`
- [x] Tests: `Email` header masked by `email` rule; unmatched rule listed
- [x] `cargo test -p db-pro-core masking` — 9 passed
