# Select

Dropdown selector with keyboard navigation, an optional label, accessible widget information, and an optional “Load more…” action.

## Usage

```rust
Select::new("connection.pool", &mut selected_index, &options)
    .theme(theme)
    .label("Cluster pool")
    .has_more(has_more)
    .load_more(&mut request_more)
    .show(ui);
```

`selected_index` is the index into `options`; when it is out of range, the trigger displays `Select an option...`. `request_more` is set when the user activates “Load more…”. Use a stable, unique `id_salt` for each selector instance.

## Layers

- `mod.rs`: public entry and API exports.
- `ui.rs` / `ui/`: egui presentation and interaction plumbing.
- `handler.rs`: selection, keyboard, popup-dismissal and dropdown-geometry decisions.
- `config.rs`: Select-specific dimensions and spacing. Shared design values remain in `crates/ui/src/tokens.rs` and `DbProTheme`.
