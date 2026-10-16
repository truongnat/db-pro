# Composite Foreign-Key Cell Picker

State: REVIEW.
Issues: truongnat/db-pro#496 and #447.
Branch: `fix/reject-composite-fk-cell-picker`.

## Goal

Prevent a composite foreign-key tuple from being offered as one scalar value in a single editable grid cell, while retaining referenced-row navigation for composite keys.

## Evidence

The cell picker selected FKs by membership in `from_columns`, queried every referenced column, then joined each row into one comma-separated option value. Selecting that option wrote a tuple string into one column.

## Scope

- Restrict single-cell FK lookup to exactly one source and one destination column.
- Keep the general FK lookup available to referenced-row navigation, which maps all key columns to filters.
- Test that composite keys do not request a scalar picker and still open referenced rows using all key fields.

## Non-goals

- No multi-column editor or composite-key selection UI.

## Acceptance

- [x] Composite FK columns remain editable as text without offering a malformed scalar picker.
- [x] Referenced-row navigation continues to constrain all composite-key fields.
- [x] Focused tests, formatting, and app check pass.
