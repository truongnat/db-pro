# Chrome Components

Small native egui chrome widgets used by DB Pro surfaces: avatars, skeleton placeholders, empty states, and compact toolbars.

## Public API

- `Avatar`, `AvatarSize`, `AvatarShape`, `AvatarStatus`: builder-style avatar with optional initials, Lucide icon, shape, size, semantic status dot, and an accessible `.access_label(...)` override. Initials are the default accessible label; otherwise it is `Avatar`.
- `Skeleton`: animated placeholder block. A width of `0.0` or less fills available space; positive widths are clamped to it. Negative/non-finite available widths become zero. Negative heights/rounding become zero; non-finite heights/rounding use defaults.
- `EmptyState`: centered icon/title/description block with an optional action button response.
- `Toolbar` and `toolbar_button`: themed compact toolbar frame and standard icon button helper.

These names are re-exported from both `components::chrome::*` and `components::*`.

## Behavior

- Avatar initials take precedence over icons when both are configured, preserving text identity.
- Avatar status colors come from semantic `DbProTheme` tokens (`success`, `danger`, `warning`, `text_disabled`).
- Skeleton pulse alpha and shimmer geometry are calculated in `handler.rs`; `ui.rs` only allocates and paints egui primitives.
- Toolbar buttons use the shared `Button` component with the ghost/icon-small variant.

## Example

```rust
use crate::components::{toolbar_button, Avatar, AvatarSize, AvatarStatus, Skeleton, Toolbar};

Avatar::new(theme)
    .initials("DB")
    .size(AvatarSize::Md)
    .status(AvatarStatus::Online)
    .show(ui);

Skeleton::new(theme).size(160.0, 12.0).show(ui);

Toolbar::new(theme).show(ui, |ui| {
    toolbar_button(ui, "Run", lucide_icons::Icon::Play, theme)
});
```
