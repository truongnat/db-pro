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

## Owner-reported regression pass — 2026-10-19

Baseline: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; changes remain uncommitted on `main`.
Keep this existing plan as the single owner of the follow-up fixes:

- Suppress duplicate egui shell separator strokes; retain semantic subtle dividers and resize affordances.
- Restore visible Light/Dark switching, consistent settings/persistence, and theme propagation into the root UI.
- Keep shared single-line SearchInput compact inside tall containers, so Quick Open results remain visible.
- Verify with interaction/geometry regressions, native light/dark captures at the three requested sizes, and Rust gates.
- Preserve existing layout, typography, database behavior, and keyboard focus rings.

## Border follow-up after owner rejection

The owner reported the borders reappearing. The earlier separator-only change retained
explicit frame strokes; that was insufficient. Remove decorative outer strokes at the
topbar, statusbar, output panel, Agent panel, and activity rail callers. Preserve
input/focus borders and interactive resize feedback.



Provider impact: PostgreSQL and SQLite n/a; presentation-only change.
