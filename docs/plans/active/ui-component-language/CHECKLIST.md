# Checklist

- [x] Inspect #288 dependencies and existing component primitives.
- [x] Migrate value inspector mode/actions to canonical components.
- [x] Migrate Explorer search/refresh controls to canonical components.
- [x] Migrate Schema Workbench actions/confirmation to canonical components.
- [x] Add a focused guard against raw `ui.button` regressions on the primary surfaces.
- [x] Run Rust quality gates and native release build.
- [ ] Clean-code scan is fully green; inherited long Schema Workbench methods remain.
- [ ] Collect native runtime screenshots for #287/#288.
- [ ] Close #288; blocked until runtime evidence and remaining UI surfaces are covered.
- [x] Standardize Tasks view, Settings session actions, and Query Output EXPLAIN button to canonical `Button`.
- [x] Expand non-regression test in `components/mod.rs` to cover `tasks_view.rs` and `query_output_view.rs`.
