# Code API

Import from `db_pro_ui::components::code`.

```rust
CodeBlock::new("SELECT id, name FROM m_client;", theme)
    .language("sql")
    .show_line_numbers(true)
    .show(ui);
```

## InlineCode

`InlineCode::new(text: &str, theme)` accepts borrowed text. `.show(ui) -> Response` paints a non-interactive inline chip and returns its egui response.

## CodeBlock

- `CodeBlock::new(code: &str, theme)` creates a block with line numbers enabled.
- `.language(label)` sets the displayed language label; the default is `SQL`.
- `.show_line_numbers(bool)` toggles the gutter.
- `.show(ui) -> Response` paints the block and returns the outer response.

The header's Copy button copies the complete source to egui's clipboard output. Its accessible label changes between `Copy` and `Copied`; the confirmation lasts two seconds. The API does not return clipboard success because egui exposes this as an output request. Text is not syntax-highlighted or wrapped; the widget has no scrolling or line virtualization. Use a bounded parent for long content.
