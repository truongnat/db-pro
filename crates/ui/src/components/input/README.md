# Input components

Native `egui` fields for short text, secrets, search, and multiline text. The public entry point is `components::input`; rendering remains in `ui.rs` submodules and UI-independent label/counter decisions live in `handler.rs`.

```rust
Input::new(&mut name, "Table name", theme)
    .label("Name")
    .clearable(true)
    .show(ui);

PasswordInput::new(&mut password, "Password", &mut reveal, theme).show(ui);
```

`Textarea::max_chars` provides a Unicode character counter; it does not truncate user input. Password reveal is explicitly labelled for accessibility, and `Input` uses `access_label` before its visible label or placeholder.
