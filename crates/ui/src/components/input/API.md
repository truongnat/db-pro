# Input API

Import `Input`, `PasswordInput`, `SearchInput`, and `Textarea` from `db_pro_ui::components::input`.

```rust
Input::new(&mut name, "Table name", theme)
    .label("Name")
    .clearable(true)
    .show(ui);

PasswordInput::new(&mut password, "Password", &mut reveal, theme).show(ui);
Textarea::new(&mut notes, "Notes", theme).max_chars(500).show(ui);
```

## Widgets

- `Input::new(value: &mut String, placeholder, theme)` supports `.label(text)`, `.access_label(text)`, `.helper_text(text)`, `.error_text(text)`, `.leading_icon(Icon)`, `.clearable(bool)`, `.width(points)`, `.enabled(bool)`, `.id_salt(hashable)`, `.auto_focus(bool)`, and `.show(ui) -> Response`.
- `SearchInput::new(value: &mut String, placeholder, theme)` supports `.shortcut(text)`, `.width(points)`, and `.show(ui) -> Response`; its clear control clears the borrowed string.
- `PasswordInput::new(value: &mut String, placeholder, visible: &mut bool, theme)` supports `.label(text)`, `.helper_text(text)`, `.error_text(text)`, `.required(bool)`, `.width(points)`, `.id_salt(hashable)`, and `.show(ui) -> Response`. Reveal state remains caller-owned.
- `Textarea::new(value: &mut String, placeholder, theme)` supports `.label(text)`, `.min_rows(rows)`, `.max_chars(limit)`, and `.show(ui) -> Response`.
- `INPUT_MIN_WIDTH` remains public; `resolve_field_width(requested, available)` and the `input::layout` module path remain available for compatibility.

An explicit access label takes precedence over visible label and placeholder. Disabled state outranks error/focus styling; error outranks focus, then hover. Error/helper text does not alter the value. `max_chars` displays a Unicode character count but does not reject or truncate input. egui handles editing and standard keyboard navigation.

The public config aliases `INPUT_ROUNDING`, `FIELD_INNER_MARGIN_X`, `FIELD_INNER_MARGIN_Y`, and `INPUT_ICON_GAP` were removed. Replace them with `RADIUS_XS`, `SPACE_SM`, and `SPACE_XS` from `db_pro_ui::tokens`.
