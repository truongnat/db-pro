# Issue #503 — Findings

- `suggest_sensitive_columns` lowercases for detection but returns the
  original header names; users can also type any casing into the columns CSV,
  so strict equality could miss real columns.
- Unmatched rules (not unmatched headers) are the dangerous direction: a rule
  that matches nothing means the user *intended* masking that never happens.
  Headers without rules are ordinary pass-through and need no warning.
- `MaskingPreview.message` is already rendered by
  `transfer_activity_surface_view::draw_masking_result`, so appending the
  unmatched-rule names there reaches the user with no UI change.
