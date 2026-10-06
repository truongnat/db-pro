# Table Indexes Responsive Layout — Plan

State: RUNTIME_VERIFY
Branch: `main` (repository owner requires direct-main workflow)

## Goal

Keep every Indexes table column visible in the standard Table Detail viewport and make long cell values discoverable without overflowing neighboring columns.

## Evidence / baseline

- Baseline source: `crates/ui/src/table_indexes_surface_view.rs` at `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Native 1280×800 capture showed only Index Name, Indexed Columns, Method, INCLUDE, and Predicate; Status was beyond the visible edge.
- The declared fixed widths total 1,020pt before Status. The shared table gives the remaining column an 80pt minimum, yielding 1,100pt in a 928pt viewport.

## Scope

- Size the Indexes columns from the available viewport width, with a minimum table width for narrower windows.
- Truncate cell labels to their actual cell width and expose full values on hover.
- Preserve the shared table component's intended horizontal scrolling when the viewport is below the minimum width.
- Correct the singular index count label.

## Non-goals

- Change index metadata loading, provider introspection, or create/drop behavior.
- Change the shared Table geometry contract or other metadata tabs.

## Architecture and provider matrix

The Indexes pane presents the existing `UiTableInfo.indexes` metadata and issues no database operation. PostgreSQL and SQLite provider behavior is unchanged and not runtime-tested by this layout fix. The active `schema-indexes-runtime` plan continues to own create/drop and provider lifecycle evidence.

| Provider | Support for this change | Runtime evidence |
|---|---|---|
| PostgreSQL | Existing metadata presentation | Not run; deterministic fixture only |
| SQLite | Existing metadata presentation | Not run; deterministic fixture only |

## Acceptance criteria

- [x] At 928pt available width, all six columns fit, including Status.
- [x] Long index names, columns, predicates, and statuses clip to their cell and show the full value on hover.
- [x] A focused geometry regression check fails on the old 1,100pt layout and passes on the responsive layout.
- [x] Native loaded-state capture at 1280×800 reviewed.
- [ ] Review capture at 1440×900 and 1920×1080, plus empty/loading states.
- [x] No PostgreSQL or SQLite query/mutation behavior changed.
