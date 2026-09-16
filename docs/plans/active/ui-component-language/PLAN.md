# UI Component Language — #288 slice

## Goal

Reduce visible control drift on two primary native surfaces by routing their
actions and search through the existing canonical egui components.

## Scope

- Value inspector mode switching and actions in `result_grid_edit.rs`;
- Explorer search and refresh context action in `explorer_view.rs`;
- no new UI framework, token system, or component abstraction.

## Non-goals

- full `legacy.rs` removal;
- every native surface in #288;
- #287 runtime screenshot matrix and light/dark viewport qualification;
- changes to data editing semantics or provider behavior.

## Acceptance

- inspector modes use `SegmentedTabs`;
- inspector actions use canonical `Button` variants/sizes;
- Explorer search uses `SearchInput` and refresh menu uses `ctx_menu_item`;
- existing behavior remains unchanged;
- Rust quality gates pass; UI runtime evidence remains explicitly pending per #287.
