# Checklist — Explorer First-Tab Visual Redesign

## Planning
- [x] Review Explorer source inventory and research report.
- [x] Confirm design scope is visual-only; no F1–F11 feature additions.
- [x] Create feature branch and lifecycle plan directory.

## Implementation
- [x] Capture and inspect the Explorer loading baseline before editing.
- [x] Add a compact Connections hierarchy label below the filter.
- [x] Highlight the active connection and expand the connecting row.
- [x] Preserve existing action pipeline and provider guards.
- [x] Inspect empty and loading states at 1280×800.

## Verification
- [ ] Capture exact 1440×900 and 1920×1080 Explorer viewports; current host caps window height.
- [ ] Capture connected-tree and error states with a usable provider fixture.
- [x] Visually inspect available captures and document display-limit/error-state gaps.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo check --workspace`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo test --workspace` (1327 passed, 41 ignored).
- [x] Run `cargo build --release --locked -p db-pro-native`.
- [x] Record exact command outcomes and remaining findings.

## Wheel scrolling regression
- [x] Reproduce wheel input against a long Explorer tree in a headless `draw_sidebar` test.
- [x] Keep the scrollable child UI on the SidePanel's actual egui layer.
- [x] Verify the tree offset advances from mouse wheel input without launching the native app.
- [ ] Owner verifies wheel scrolling in a real build.
