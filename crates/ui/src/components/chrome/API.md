# Chrome API

The component types are re-exported from `db_pro_ui::components` and `db_pro_ui::components::chrome`.

```rust
Avatar::new(theme).initials("DB").status(AvatarStatus::Online).show(ui);
Skeleton::new(theme).size(180.0, 14.0).show(ui);
EmptyState::new(Icon::Database, "No connections", "Add a database to get started", theme).show(ui);
Toolbar::new(theme).show(ui, |ui| toolbar_button(ui, "Run", Icon::Play, theme));
```

## Widgets

- `Avatar::new(theme)`; `.initials(&str)`, `.icon(Icon)`, `.size(AvatarSize::{Sm,Md,Lg})`, `.shape(AvatarShape::{Circle,Rounded})`, `.status(AvatarStatus::{Online,Busy,Away,Offline})`, `.access_label(&str)`, `.show(ui) -> Response`. Initials take precedence over the icon. Accessibility name is the nonblank override, otherwise initials, otherwise `Avatar`.
- `Skeleton::new(theme)`; `.size(width,height)`, `.rounding(radius)`, `.shimmer(bool)`, `.show(ui) -> Response`. Nonpositive width fills available width; positive width is capped to it. Invalid dimensions use safe defaults. Reduced motion freezes the placeholder and stops repaint scheduling.
- `EmptyState::new(icon,title,description,theme)`; `.action(label)` optionally adds a Button; `.show(ui) -> Option<Response>` returns `None` without an action and the button response when one is present.
- `Toolbar::new(theme).show<R>(ui, add_contents) -> R` renders a horizontal compact frame and returns the closure value.
- `toolbar_button(ui, tooltip, icon, theme) -> Response` returns the shared Button response; `tooltip` is also its accessible name.

These are layout widgets, not application state owners. Avatar status is visual metadata; Skeleton has no progress value; EmptyState does not decide when data is empty; Toolbar does not dispatch actions. The prior config-only radius aliases were internal and removed; use shared tokens when composing custom styles.
