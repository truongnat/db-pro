# Button API

Import `Button`, `ButtonGroup`, `ButtonSize`, and `ButtonVariant` from `crate::components` or `crate::components::button`. Construct a new Button during each egui render with the current `DbProTheme`; call `.show(ui)` once and use the returned `egui::Response` for the action.

```rust
use crate::components::{Button, ButtonSize, ButtonVariant};

let response = Button::new(theme)
    .text("Run query")
    .icon(lucide_icons::Icon::Play)
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Sm)
    .enabled(can_run)
    .loading(is_running)
    .tooltip("Execute the current statement")
    .show(ui);

if response.clicked() {
    run_query();
}
```

## Builder reference

| Method | Input | Effect |
|---|---|---|
| `Button::new(theme)` | `DbProTheme` | Creates an enabled, focusable, default-size, default-variant button. |
| `.text(text)` | `Into<Cow<str>>` | Sets the visible label and its accessible-name fallback. |
| `.icon(icon)` | `lucide_icons::Icon` | Places an icon before text. Icon-only buttons must also set `.access_label(...)`. |
| `.variant(variant)` | `ButtonVariant` | Chooses the visual treatment. |
| `.size(size)` | `ButtonSize` | Chooses height, typography, icon size, padding, and minimum width. |
| `.enabled(bool)` | `bool` | Disables activation and focus when false. |
| `.loading(bool)` | `bool` | Shows a non-activating loading cue when enabled. With `theme.reduce_motion`, the cue remains static. Disabled takes precedence over loading. |
| `.full_width(bool)` | `bool` | Uses the current egui available width when true. |
| `.left_aligned()` | none | Aligns icon/text to the leading padding. |
| `.access_label(label)` | `Into<Cow<str>>` | Sets a stable accessible name; required for icon-only buttons. |
| `.tooltip(text)` | `Into<Cow<str>>` | Shows a tooltip on the response. |
| `.focusable(bool)` | `bool` | Allows or removes keyboard focus for an enabled button. |
| `.accessible_name()` | none | Returns the resolved accessible name as `String`. |
| `.show(ui)` | `&mut egui::Ui` | Renders once and returns `egui::Response`. |

Builder methods consume and return `Self`, so they can be chained. `text`, `access_label`, and `tooltip` accept borrowed or owned strings for the builder's lifetime.

## Variants and sizes

| `ButtonVariant` | Intended use | Rest treatment |
|---|---|---|
| `Default` | Main action | Accent fill and on-accent text. |
| `Secondary` | Supporting action | Hover-surface fill. |
| `Outline` | Bordered alternative | Transparent fill and default border. |
| `Ghost` | Low-emphasis action | Transparent fill without border. |
| `Destructive` | Destructive action | Danger fill. |
| `Link` | Inline text action | Transparent fill; underline on hover or focus. |

`ButtonSize` has `Sm`, `Default`, `Lg`, `Icon`, and `IconSm`. Preset dimensions live in `crate::tokens::component::button`; use those tokens as the source when adjusting a shared button size. Icon-only controls still require a meaningful hit target and accessible label. `IconSm` is a compact desktop preset.

## State and response contract

The core button contract resolves simultaneous states in this order: disabled, loading, active, focus, hover, default. Disabled+loading displays disabled; enabled+loading displays the spinner and wait cursor. Disabled and loading responses cannot be clicked. Enabled interactive responses carry egui click, hover, and focus information; the caller handles the resulting action.

The accessible-name fallback order is `access_label` → visible text → tooltip → `"Button"`. For icon-only buttons, set a nonblank `access_label` explicitly even when a tooltip exists. `show()` enforces this in debug and release builds; an invalid icon-only call panics rather than rendering an unnamed action.

`DbProTheme::reduce_motion` carries the current accessibility preference to Button. The application sets it from preferences each frame. Hover/press changes become immediate and the loading cue stops animating when enabled. Filled variants ask `DbProTheme::text_on_solid` for a readable label color against the current fill at every hover step.

```rust
Button::new(theme)
    .icon(lucide_icons::Icon::X)
    .access_label("Close panel")
    .tooltip("Close")
    .variant(ButtonVariant::Ghost)
    .size(ButtonSize::Icon)
    .show(ui);
```

## Grouping

`ButtonGroup::new(theme).show(ui, |ui| { ... })` lays out buttons horizontally with the shared `SPACE_XS` gap and returns the closure result. The theme argument is retained for API compatibility; individual buttons still receive their own current theme.

```rust
ButtonGroup::new(theme).show(ui, |ui| {
    Button::new(theme).text("Apply").show(ui);
    Button::new(theme).text("Cancel").variant(ButtonVariant::Ghost).show(ui);
});
```

`ButtonPalette` and `SizeTokens` remain exported from `components::button` for existing advanced callers. `SizeTokens::from_size(size)` returns preset dimensions and `calculate_width(...)` applies content/full-width sizing. `ButtonPalette::from_variant(variant, theme)`, `disabled(theme)`, `loading_colors(...)`, and `resolve_state(hover)` expose the current visual calculations. Product screens should normally use `Button` instead of painting from these values themselves.
