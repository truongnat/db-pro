# Checklist — rs-ui Runtime Integration

- [x] Audit DB Pro host, state, command bridge, and shell entry point.
- [x] Audit rs-ui workspace crate boundaries and renderer/window choices.
- [x] Add local path dependencies for the minimal core/runtime crates.
- [x] Add a renderer-independent shell layout adapter using rs-ui `UiTree`.
- [x] Feed rs-ui shell geometry into the existing sidebar host.
- [x] Route sidebar pointer resizing and focused arrow-key adjustments through rs-ui `Resizable` behavior while retaining DB Pro's persisted width as source of truth.
- [x] Verify the egui-to-rs-ui pointer delta path with an app-level drag test.
- [x] Route sidebar wheel deltas through rs-ui `ScrollState`; sync viewport/content and scrollbar drag offsets back from the existing egui painter.
- [x] Keep the existing sidebar wheel-scroll regression test green through the rs-ui adapter.
- [x] Run targeted tests and workspace quality gates.
- [x] Capture native app empty/loading/error/normal states where the current display allows.
- [ ] Complete native app smoke for opening a connection, browsing schema, running a query, switching tabs, scrolling results, resizing panes, and keyboard focus.
- [ ] Capture the required 1920×1080 viewport; current display limits the window size.
- [x] Route tab and close-button activation, plus sidebar/query-dock split resizing, through rs-ui `Pressable`/`Resizable`; retain existing DB Pro state and action handlers.
- [ ] Verify splitter/tab focus and accessibility semantics in the native app.
- [ ] Verify splitter/tab focus and accessibility semantics in the native app.
- [ ] Resolve local dependency pinning for CI/release builds.
- [ ] Independently review and account for wgpu/glow coexistence before renderer migration.
