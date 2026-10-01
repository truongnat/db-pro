# Button design

## Purpose and boundaries

Button presents an action at the point where a user can take it. The control owns its visual variants, interaction state, measured layout, accessibility metadata, and loading display. The caller owns the action performed after `Response::clicked()` and the lifetime of `enabled` and `loading` values.

The component receives the current `DbProTheme` from its caller. Shared color roles, spacing, radius, type sizes, and button size presets come from `theme.rs` and `tokens/`. `config.rs` contains only Button-specific values. A theme change reaches Button on the next render because the caller constructs it with the current theme.

## UI design

The default variant uses the accent fill so one main action is easy to locate. Secondary, Outline, and Ghost reduce visual weight for supporting actions; Destructive uses the danger role; Link is text-only with a hover/focus underline. Each variant uses semantic theme colors, so light and dark modes keep the same meaning. The palette is resolved in `handlers/palette.rs`, while `ui/button.rs` paints it.

Sizes are fixed presets from the core button contract: small, default, large, icon, and compact icon. The control measures text with egui, adds the preset padding, and keeps at least the preset width. `full_width(true)` fills the current UI region. Text and icons are centered by default; `left_aligned()` starts content at the leading padding. The rounded surface follows the shared `RADIUS_BUTTON` token. `ButtonGroup` uses the shared `SPACE_XS` gap so adjacent 2-point focus rings do not overlap.

The control distinguishes rest, hover, press, focus, loading, and disabled states. Hover blends fill and stroke; press scales the painted rect; focus keeps a visible ring. Loading shows a spinner and wait cursor. Disabled keeps muted text and a quiet bordered surface. These states make the action and its availability visible in dense IDE screens without adding heavy decoration.

## Logic design

1. The builder stores caller inputs; `show(ui)` selects size tokens and applies the core state precedence: disabled before loading, then active, focus, hover, and default.
2. Loading allocates a hover-only response, measures the optional label, paints the variant's loading surface and spinner, and returns a wait cursor. It cannot activate the caller's action.
3. Interactive rendering resolves the variant palette or disabled palette, measures the icon/text galley, and allocates the response. Enabled controls use egui click and focus; disabled controls allocate hover-only sense.
4. egui supplies hover, pointer-down, and focus signals. The core button contract resolves their precedence; animation helpers produce hover and press progress. The handler blends colors and calculates content positions. The UI paints the surface, content, optional link underline, and focus ring, then attaches the tooltip.
5. The caller reads the returned `Response` and decides what action to perform. Button does not mutate application state.

The accessible-name order is explicit label, visible text, tooltip, then `"Button"`. Icon-only rendering requires a nonblank explicit `access_label` in all builds; tooltip text is only a fallback. Widget metadata follows the enabled state. Focusability can be disabled independently with `focusable(false)`.

Default and Destructive text colors use the core theme's `text_on_solid` decision against each frame's resolved fill. This keeps contrast at normal-text level as hover blending changes the fill. The application's existing Reduce motion preference is carried by `DbProTheme`; Button uses immediate hover/press states and a static loading cue when it is set.

## Performance and review points

Style is derived from the supplied theme during rendering; there is no separate cache or stylesheet parser. Text is laid out once per interactive render and reused for measurement and painting. Loading measures its optional text once, then paints from that galley. Animation state is keyed by the egui response ID so neighboring buttons do not share hover/press progress.

When reviewing a change, check all variants in light and dark themes, text and icon-only content, narrow and full-width layouts, and the combinations disabled+loading and focused+hovered. Inspect whether the change belongs to a shared token, a Button-specific config value, a pure handler decision, or egui painting. Keep `Button`, `ButtonGroup`, `ButtonVariant`, `ButtonSize`, `ButtonPalette`, and `SizeTokens` available through `mod.rs`.

The acceptance check also covers text/focus contrast, meaningful accessible names and Button role/state, Tab plus Space/Enter activation, minimum target size, reduced motion, and the native accessibility tree. Loading must remain understandable without spinner motion. Review these with the same theme and state combinations as the visual check; source inspection alone cannot close the runtime checks.

## File map

Code comments at the UI/handler boundary trace the flow described above: inputs and egui events, handler decisions, painted outcome, and returned response. They explain the state precedence or layout reason where a future change could break the contract.

- `mod.rs` is the component entry point and public exporter.
- `ui/button.rs` owns the Button builder, egui allocation, rendering, tooltip, and accessibility metadata.
- `ui/group.rs` owns the ButtonGroup layout.
- `handlers/size.rs` maps size presets and calculates width.
- `handlers/palette.rs` maps semantic theme roles to variants and blends interaction colors.
- `handlers/mod.rs` declares public variants and pure content-position calculations.
- `config.rs` owns only the Link underline thickness.
