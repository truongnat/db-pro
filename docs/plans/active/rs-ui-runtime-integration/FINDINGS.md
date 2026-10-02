# Findings — rs-ui Runtime Integration

Baseline: `db-pro@c0c1f5525b20a810913d1eee13c7ee2dd15b6664` before this patch. Current implementation commit: `42f14e520fa1e8bb280c1ec0399d3f523d4792da`. The sibling rs-ui checkout has commit baseline `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`, source tree observed during verification `b66c61ac255ac4bcd1eb2c3d10e11abc2011886d`, and uncommitted changes.

## Evidence

- DB Pro UI entry is `DbProApp::update` and calls the existing shell lifecycle in `crates/ui/src/app_lifecycle.rs`.
- The sidebar is hosted by egui `SidePanel` in `crates/ui/src/sidebar_surface_view.rs`.
- DB Pro's service and domain code are behind the existing UI command/event bridge, documented in `docs/architecture/system-overview.md`.
- rs-ui's workspace separates `ui-core`, `ui-runtime`, `ui-renderer`, and `ui-window`; runtime owns retained layout and behavior while the renderer uses wgpu.
- The sibling rs-ui working tree has uncommitted changes. The new local dependency therefore consumes a mutable checkout.
- The sidebar resize handle uses egui for pointer capture/focus detection, then calls rs-ui `Resizable` APIs for pointer delta, bounds, keyboard movement, and slider semantics. An app-level egui pointer-drag test verifies the action reaches the existing workspace width setter.
- At DB Pro source commit `948654524a6cd8a350a3eb972df33f2a5a1cd9ed`, query-output bottom/right splitters use rs-ui `Resizable` with inverted pointer coordinates to preserve existing drag direction and DB Pro size setters. Workspace tab selection nodes use rs-ui `Pressable` with `Tab`/`TabList` semantics; closed nodes are removed after the render pass.
- Sidebar wheel deltas are routed through rs-ui `ScrollState`; the retained offset drives egui content paint, while egui scrollbar interactions sync their resulting offset and layout extent back into the adapter.
- The shell and tabs continue to paint through egui. Tab and close-button activation are normalized through `Pressable`; native accessibility/focus behavior has not been verified.

## Failure scenario / severity

- P2 integration risk: a future renderer cutover from eframe's current glow path needs separate startup, accessibility, and capture validation. This change does not perform that cutover.
- P2 reproducibility risk: path dependencies resolve against mutable sibling files and are not available on clean CI machines.
- No proven defect in DB Pro business or database behavior was found in this integration audit.

## Decision

Use rs-ui core/runtime as the shell layout authority through a thin adapter, retaining egui for host input and painting. Do not depend on `ui-renderer` or `ui-window` in the first slice.

## Provider impact and runtime testability

PostgreSQL: N/A; SQLite: N/A. This change does not touch database behavior. Shell screenshots from the earlier layout pass exist at 1280×800 and a constrained 1440×838 logical capture; 1920×1080 was not captured. Native product smoke is still pending. The adapter unit and app-level drag tests are not OS-level input proof.
