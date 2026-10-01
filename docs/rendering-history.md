# egui rendering history

## Local history

The DB Pro Git history does not contain egui implementation commits. The project resolves egui crates from crates.io; the tracked source revision used for this audit is `0f1b34dbb147ce130a6ae5899c2523f816adb0b`, with egui family crates at `0.29.1`. Historical renderer decisions below come from the upstream egui PRs and changelogs, then are checked against the exact `0.29.1` registry source.

## #2071 — gamma-space color operations (2022)

Upstream PR [#2071](https://github.com/emilk/egui/pull/2071), merged as `4ac1e28` on 2022-09-24, removed linear framebuffer blending and linear blending in the pixel shader. The stated reason was text quality: linear blending made low-contrast and colored text look faint or develop gray fringes, and made dark-on-bright and bright-on-dark strokes appear to have different weights. The PR retained sRGB-aware texture sampling, then encoded sampled texture values before gamma-space multiplication. It also explicitly asked integrations to disable sRGB framebuffer blending.

The trade-off was intentional: improve grayscale coverage and text appearance while changing image tinting and shadows. This decision is in the current 0.29.1 Glow shader and blend setup. It is not evidence that gamma-space compositing is universally colorimetrically correct; it is evidence that it was selected to improve egui's text and 2D UI behavior.

## #7311 — texture filtering in gamma space (2025)

Upstream PR [#7311](https://github.com/emilk/egui/pull/7311) landed in egui 0.32.0 (2025-07-10), after DB Pro's locked 0.29.1. The [egui_glow changelog](https://github.com/emilk/egui/blob/main/crates/egui_glow/CHANGELOG.md) describes it as improving texture filtering by doing it in gamma space. Therefore DB Pro's Glow texture interpolation behavior predates that adjustment. `predictable_texture_filtering` is a later WGPU renderer option and is not present in DB Pro's active Glow integration.

## #8283 — reconsider linear versus sRGB blending (2026)

Upstream [issue #8283](https://github.com/emilk/egui/issues/8283) reopens the color-space trade-off. The issue argues for comparing linear-light compositing against the gamma-space behavior and raises consistency concerns when texture filtering, alpha premultiplication, and blending happen in different spaces. It is an open discussion, not a merged reversal of #2071. DB Pro's current source must therefore be described as 0.29.1's gamma-space rendering behavior; a local alternative renderer should be experimentally compared before changing the default.

## Text path context

Upstream [issue #2639](https://github.com/emilk/egui/issues/2639) documents that egui's text path rounds glyph cursors and glyph geometry to the physical pixel grid to reduce linear-filtering artifacts. It also records the trade-offs for small fonts and monospace spacing. In 0.36.1 upstream changed the font backend to Skrifa plus `vello_cpu`, adding hinting; 0.36.2 also fixes thin angled rectangles. These are relevant improvements, but the 0.29-to-0.36 update is a breaking egui API migration for this codebase, not a small renderer option change.

## Upgrade experiment in this checkout

The two app manifests were temporarily changed to `0.36.2`, then `cargo check -p db-pro-native` was run. The resolver succeeded, but compilation reported 577 API errors in `db-pro-ui`: `Rounding` was removed, panel types and style APIs changed, margins/shadows changed representation, and current `eframe::App` requires `ui` instead of the old `update` entry point. The manifests and `Cargo.lock` were restored to the known `0.29.1` baseline. This ruled out applying the suggested upgrade as the first small rendering patch; it does not rule out a separately planned full migration.
