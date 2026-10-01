# Badge design

Badge is an intrinsic-width, non-interactive label for status and metadata. `ui.rs` measures the text, selects standard or compact metrics, allocates one hover-sense rectangle, and paints the themed fill, border, optional leading content, and label. The handler maps semantic variants to `DbProTheme` colors and computes leading-content sizing/placement.

When both a dot and icon are requested, the dot takes priority. The badge text is attached to egui label metadata; there is no focus, keyboard action, or animation. Shared radius and stroke values come from `tokens.rs`; component config keeps only Badge-owned dimensions.

Rendering cost is proportional to label text measurement and constant painting work. Width follows the full text, so long labels can dominate a narrow row; callers should shorten or wrap the surrounding layout rather than assume Badge truncates text. Semantic colors must remain distinguishable in both light and dark themes.
