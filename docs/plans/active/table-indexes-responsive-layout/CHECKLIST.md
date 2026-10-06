# Table Indexes Responsive Layout — Checklist

## Planning
- [x] Capture the failure at a deterministic viewport.
- [x] Record the width budget and scope.

## Implementation
- [x] Derive Indexes column widths from the viewport.
- [x] Keep Status readable at standard width and retain horizontal scrolling below the minimum table width.
- [x] Use cell-width truncation with hover disclosure.
- [x] Correct singular/plural index count.

## Verification
- [x] Geometry regression test observed failing before the fix and passing afterward.
- [x] Release native capture build succeeded.
- [x] Loaded-state native capture reviewed at 1280×800.
- [ ] 1440×900 / 1920×1080 and empty/loading captures reviewed.
- [x] `git diff --check` passed for changed Indexes and capture files.
- [x] `STATUS.md` matches the plan state (`RUNTIME_VERIFY`).

## Metadata header and modal removal — 2026-10-06
- [x] Match Indexes/Structure filter input/title/total font and vertical alignment.
- [x] Remove redundant modal and opening actions from Indexes and Structure.
- [x] Keep full-value hover disclosure.
- [x] Change index-name interaction regression to assert passive metadata.

## Constraints follow-up — 2026-10-06
- [x] Shared input/label sizing and semantic border colors.
- [x] Parent tab component reused for right-aligned category filters.
- [x] Type icon/badge spacing uses SPACE_SM.
- [ ] Full loading/error/empty and interactive category/search runtime matrix.

## Dependencies follow-up — 2026-10-06
- [x] Shared header input/label typography, height, borders and clear control.
- [x] Parent-style category controls aligned right.
- [x] Direction icon/badge gap uses SPACE_SM.
- [ ] Interactive filters/navigation and loading/error/empty capture matrix.
