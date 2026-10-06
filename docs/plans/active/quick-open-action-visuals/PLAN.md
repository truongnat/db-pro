# Quick Open and borderless action states

State: RUNTIME_VERIFY. Baseline: `57cea2bf746e132564cde952cc06d0d04d796aa0`.

## Goal

Make selected Quick Open scope text use the selected foreground, and remove decorative strokes from action controls and selected Quick Open rows. Keep hover fill, press feedback, and the keyboard focus ring.

## Scope

- Fix selected scope-filter text in both light and dark themes.
- Remove selected-row outlines in Quick Open and hover/active strokes from its scope filters.
- Keep Button action variants borderless in rest, hover, loading, and disabled states.
- Add regression coverage and native capture hooks for the affected surfaces.

## Non-goals

- No new user-facing feature, database behavior, or renderer architecture change.
- No change to contrast-driven `text_on_solid`; black text on bright dark-theme accent fills remains intentional.
- No removal of the keyboard focus ring or input-field borders.

## Acceptance

- Selected filter text matches `visuals.selection.stroke.color` in light and dark themes.
- Action and selected-row decoration has no stroke; selection/hover fill remains legible.
- Button stroke stays `Stroke::NONE` during hover animation, loading, and disabled states.
- Native Quick Open and Button Gallery captures confirm the result at supported viewport sizes.
- Full requested 1440×900 and 1920×1080 logical heights remain pending because this display clamps height to 838 points.

Provider impact: PostgreSQL and SQLite n/a; presentation-only change.
