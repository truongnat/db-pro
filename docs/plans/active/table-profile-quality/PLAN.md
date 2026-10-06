# Table Profile Quality — Plan

State: RUNTIME_VERIFY
Branch: `main` (repository owner requires direct-main workflow)

## Goal

Make the Table Detail Profile tab's sample summaries truthful, readable, and compact at narrow and wide window sizes.

## Evidence / baseline

- Current implementation: `crates/ui/src/table_profile_surface_view.rs` computes summary values from the loaded `UiQueryResult` and renders an eight-column egui grid.
- Observed screenshot: the `created_at` sample is labeled `Enum-like`; Min and Max show different timestamp formats; the grid's intrinsic column widths leave large gaps.
- Source failure cases: numeric extrema use string ordering; a result with columns but zero rows is rendered as `100% full` / `Constant` instead of an empty state; the grid only scrolls vertically.
- Existing coverage: `test_profile_result_numeric_and_uniqueness` exercises basic numeric summaries and distinct counts.

## Scope

- Use exact decimal ordering for numeric Min/Max without converting through `f64`.
- Format recognized date/time extrema consistently while preserving any displayed offset.
- Replace inferred `Enum-like` / `Standard` quality labels with neutral patterns explicitly based on the loaded sample.
- Render a dedicated no-rows state and make the profile grid scroll in both directions.
- Reduce unnecessary grid spacing and bound long cell values with the full value available on hover.

## Non-goals

- Change query paging, provider result decoding, or the scope of the loaded sample.
- Change average/sum precision or their existing `f64` representation.
- Add a new chart, new database request, or provider-specific behavior.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes; consumes the existing UI result | N/A for this presentation-only change; live provider UI not exercised |
| SQLite | yes; consumes the existing UI result | N/A for this presentation-only change; live provider UI not exercised |

## Architecture

The Profile pane remains a presentation-only consumer of `UiQueryResult` in `crates/ui`. It does not issue a `UiCommand` or alter provider/runtime behavior. Numeric ordering uses the existing `bigdecimal` dependency; date/time display uses the existing `chrono` dependency.

## Acceptance criteria

- [x] Numeric extrema compare as exact decimals (`2` precedes `10`) and preserve source text.
- [x] Recognized date/time extrema use one readable display format; timezone offsets remain visible.
- [x] Pattern labels do not infer enum semantics from a small distinct count.
- [x] A loaded result with columns and zero rows presents a clear empty state.
- [x] Long values remain discoverable, and both horizontal and vertical grid scrolling are available.
- [x] Existing table/header theme tokens remain the only color source.
- [x] Runtime capture and automated verification status are reported honestly; visual matrix and tests remain pending.

## Owner follow-up scope — 2026-10-06
Reuse shared Table for width/clip behavior, measure actual content widths for complete numeric cells, preserve horizontal overflow, and account for Structure badges. Shared filter header/modal removal for Indexes/Structure/Foreign Keys is tracked in the existing Table Indexes Responsive Layout plan. Full UI release gate remains failed by two connection-form tests with baseline attribution unverified.
