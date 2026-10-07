# Select API

Import with `use db_pro_ui::components::Select;`.

```rust
Select::new("connection.pool", &mut selected_index, &options)
    .theme(theme)
    .label("Cluster pool")
    .has_more(has_more)
    .load_more(&mut request_more)
    .show(ui);
```

## Builder and state

- `Select::new(id_salt, selected, options)` receives a stable unique ID salt, a mutable `usize` index, and a borrowed list of `String` options.
- `.theme(theme)` supplies component theme values; `.label(text)` adds a visible and accessible name.
- `.has_more(bool)` enables the extra paging row. `.load_more(&mut bool)` receives a request when that row is activated.
- `.size(SelectSize::Default | SelectSize::Sm)` controls trigger dimensions. Default preserves the form height; Sm shares Button's compact height, font, icon and padding tokens.
- `.variant(SelectVariant::Outline | SelectVariant::Ghost)` chooses bordered form chrome or quiet toolbar chrome. Ghost retains hover, open and focus feedback.
- `.width(width)` sets trigger width; `.theme(theme)` supplies semantic colors.
- `.show(ui)` returns the trigger `egui::Response`; selection and load-more remain mutable state outputs.

The selected index must refer to the provided options. An invalid index is not clamped and displays “Select an option...”. Keyboard movement stops at either end of the loaded list. An empty list can still show the paging row. The component does not fetch or append options itself and does not virtualize large lists. Popup geometry is constrained by the available screen region.

## Layers

`ui.rs`/`ui/` own egui rendering and interaction plumbing; `handler.rs` owns selection, keyboard, dismissal, and geometry decisions; `config.rs` owns Select dimensions. No public token aliases were found in Select config.
