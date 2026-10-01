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
- `.show(ui)` draws the control and returns `()`; the selected index and load-more flag are the output state.

The selected index must refer to the provided options. An invalid index is not clamped and displays “Select an option...”. Keyboard movement stops at either end of the loaded list. An empty list can still show the paging row. The component does not fetch or append options itself and does not virtualize large lists. Popup geometry is constrained by the available screen region.

## Layers

`ui.rs`/`ui/` own egui rendering and interaction plumbing; `handler.rs` owns selection, keyboard, dismissal, and geometry decisions; `config.rs` owns Select dimensions. No public token aliases were found in Select config.
