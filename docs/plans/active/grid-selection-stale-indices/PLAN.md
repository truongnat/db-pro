# Grid Selection Recovery After Projection Changes

State: REVIEW.
Issues: truongnat/db-pro#480 and #505.
Branch: `fix/grid-selection-stale-indices`.

## Goal

Recover keyboard selection when an option list or filtered grid projection shrinks while a selection index refers to the previous contents.

## Scope

- Clamp stale selection index in the select handler; empty option lists return index 0.
- For stale grid rows, ArrowDown chooses the first visible row and ArrowUp the last; other navigation keys safely resolve from the first visible row.
- Update select API docs and add regression tests for both interaction paths.

## Acceptance

- [x] Select keyboard navigation recovers immediately from stale indexes.
- [x] Grid arrow navigation reaches the relevant projection edge instead of skipping a row.
- [x] Existing navigation behavior, focused tests, formatting, and clean-code scan pass.
