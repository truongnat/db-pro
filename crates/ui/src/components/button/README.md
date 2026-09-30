# Button

Native egui button used for primary actions, secondary actions, destructive actions, icon controls, and link-style controls. The builder preserves the existing public API and supports loading, disabled, full-width, left-aligned, tooltip, and accessibility-label states.

## Usage

```rust
if Button::new(theme)
    .text("Save")
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Sm)
    .loading(is_saving)
    .show(ui)
    .clicked()
{
    save_changes();
}
```

For icon-only buttons, provide a visible label or an explicit `access_label(...)`; a tooltip is a secondary fallback, not a substitute for a stable accessible name. `ButtonSize::IconSm` uses compact desktop-IDE dimensions; use larger presets where touch-sized targets are required.

## Public API

- `Button`: builder-style egui widget.
- `ButtonGroup`: horizontal container that keeps adjacent buttons visually compact.
- `ButtonVariant`: `Default`, `Secondary`, `Outline`, `Ghost`, `Destructive`, and `Link` styles.
- `ButtonSize`: `Sm`, `Default`, `Lg`, `Icon`, and `IconSm` size presets.
- `ButtonPalette` and `SizeTokens`: pure style/size calculations used by tests and advanced internal callers.

## Behavior

- Disabled buttons allocate hover-only sense, use muted semantic colors, and do not become focusable.
- Loading buttons allocate hover-only sense, show the wait cursor, and keep the current label/spinner layout with variant-appropriate text contrast.
- Link buttons underline their label on hover or keyboard focus; `ButtonGroup` intentionally uses a compact 1-point gap to keep adjacent actions visually grouped.
- Interactive buttons preserve hover/press animation, focus ring painting, tooltip display, and accessible-name fallback order: explicit access label, visible label, tooltip, then `"Button"`.
- `full_width(true)` uses the currently available egui width; otherwise width remains the larger of the size preset default and measured content plus horizontal padding.

## Layers

- `mod.rs`: public entry and API exports.
- `ui.rs`: egui allocation, animation signals, painting, focus ring, tooltip, and accessibility widget info.
- `handler.rs`: pure size, palette, width, and content-position calculations.
- `config.rs`: Button-owned dimensions and layout constants.
