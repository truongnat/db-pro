# Issue #503 — Mask rule lookup is case-sensitive and silent

## Root cause
`mask_row` matched profile rules to headers with `c.column == col`. A DB
header `Email` against a profile rule `email` (suggest already lowercases)
matched nothing, so the column exported raw with no signal.

## Fix
- Rule lookup uses `eq_ignore_ascii_case` in `mask_row` and in the new
  `unmatched_mask_rules` helper.
- Visibility: `unmatched_mask_rules(headers, profile)` returns the profile
  columns matching no header; `preview_masking` stores them on
  `MaskingPreview.unmatched_rules` (serde-defaulted) and appends
  "· N rule(s) match no column: …" to `preview.message`, which the UI already
  renders.
- `mask_transfer_batch` emits `tracing::warn!` listing unmatched rules so the
  export path is not silent either.
