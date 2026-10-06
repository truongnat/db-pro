# Table Profile Quality — Findings

## P2 — Pattern label overstates what a page sample proves

- Evidence: the prior screenshot marks `created_at` as `Enum-like` based only on four distinct values in six loaded rows.
- Root cause: the renderer infers a category from a small distinct-count threshold, regardless of type or sample size.
- Decision: show neutral sample patterns (`Unique`, `Constant`, `Repeated`, `Varied`) and rename the column to `Pattern`.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 native fixture capture reviewed.

## P2 — Timestamp values have inconsistent presentation

- Evidence: the prior screenshot shows a SQL-style timestamp for Min and an RFC3339 value for Max in the same column.
- Root cause: raw cell strings are rendered unchanged.
- Decision: normalize recognized date/time strings for display and retain offsets; leave unrecognized values intact.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 native fixture capture reviewed.

## P2 — Numeric extrema are ordered lexicographically

- Evidence: source at baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`, `table_profile_surface_view.rs:87-90`, selected Min/Max with string ordering.
- Failure example: values `2` and `10` can produce Min `10` and Max `2`.
- Decision: compare finite numeric strings using `BigDecimal`, then keep the original value for display. If a numeric cell cannot be parsed as a decimal, omit extrema rather than silently reverting to string ordering.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; automated regression coverage pending.

## P2 — Empty sample presents false completeness/pattern signals

- Evidence: a present result with columns and no rows reaches the grid; null-rate arithmetic treats it as zero nulls.
- Decision: show a dedicated no-rows state before profiling.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 native fixture capture reviewed.

## P2 — Long cell values can push columns apart or beyond the viewport

- Evidence: the screenshot's widest sample values expand the Min/Max grid columns; the grid only scrolls vertically.
- Decision: add a padded surface, group column name/type and Min/Max values, collapse Avg/Sum when the loaded sample has no numeric summaries, and ellipsize long values with full-value hover disclosure.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 capture shows the last column filling available width. Long values still ellipsize by actual cell width; hover behavior and 1440×900 / 1920×1080 captures remain pending.

## Independent review — 2026-10-04

### P2 (inherited) — An all-null column is labeled “Varied”

- Evidence: in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`, `table_profile_surface_view.rs:291-302` falls through to `Varied` when `non_null_count == 0` and `distinct_count == 0`. For a result containing rows whose values in this column are all NULL, the label says values vary although there are no non-null values to compare. At baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`, the same data was called `Enum-like`; this is a pre-existing semantic gap, not a regression from the new pattern vocabulary.
- Recommendation: show a neutral state such as `All null` / `No values` for this case; add a fixture assertion for all-null and mixed-null columns.
- Status: fixed in current working-tree blob `e9c42b45cb03921c6c4630b5defb51bd6075dbbd`: zero non-null values now render the neutral `All null` badge, covered by `all_null_column_has_a_distinct_pattern_label`. The available runtime screenshot still has no all-null column.

### Review coverage gap — typography/hover and viewport matrix remain unverified

- Evidence: `evidence/profile-light-1280x800.png` is a loaded, light-theme, 1280×800 logical capture at 2× physical scale. The `created_at` type (`timestamp wit…`) and UUID range visibly truncate; source lines `199-204` and `281-283` provide hover disclosure, but the tooltip was not exercised. No 1440×900, 1920×1080, constrained-width, empty/loading, dark-theme, or scale-factor 1.0/1.25/1.5 capture is available.
- The screenshot shows clearer panel padding and hierarchy than the supplied original and keeps the range column inside the panel. It supports only this captured state and scale.
- Disposition: do not mark the visual/runtime gate complete until the required evidence is collected or explicitly marked unsupported/not run.

## Profile width / Structure badge clipping — 2026-10-06
- Baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus existing uncommitted changes.
- P2 Profile uses intrinsic-width egui Grid columns and leaves unused space on wide screens. Reuse shared Table fixed-column scaling; horizontal overflow scrolls on narrow screens, and existing full-value hover disclosure remains.
- P2 shared Table clips exactly to the padded content rectangle; centered 1px badge strokes extend 0.5px beyond their allocation and lose the leading border. Expand the clip by 1px into padding, bounded by the actual cell rectangle. The fix applies to Structure Key/Nullable and other shared table badges without disabling clipping.
- No query/provider calculation changes or new dependencies. Added deterministic Structure capture route solely for visual verification.

## Dynamic content widths — 2026-10-06
- Owner rejected percentage/minimum widths that still truncated Avg/Sum. Profile now measures headings and rendered values with the same egui font styles. Shared TableColumn::content_width includes the Table cell padding; the existing layout grows columns proportionally only when spare width exists and preserves measured widths when scrolling is required.
- Numeric summary no longer truncates to 20 characters; range and summary labels extend within measured cells. Full-range values, percentages/counts and long numeric summary determine minimum widths.
- Added actual render regression for a long numeric summary, asserting complete text and containment by the cell clip; added Profile-only numeric capture fixture to expose the sixth column.
