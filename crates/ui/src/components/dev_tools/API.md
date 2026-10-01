# DevTools API

Import widgets from `db_pro_ui::components::dev_tools`.

```rust
TerminalBlock::new("Ready\nConnected to localhost", theme)
    .title("Connection check")
    .show(ui);

ProgressRing::new(0.72, 24.0, theme).show(ui);
```

## TerminalBlock

- `TerminalBlock::new(output: &str, theme)` stores borrowed output text.
- `.title(title: &str)` sets the header title; absent title uses `Terminal Output`.
- `.show(ui) -> Response` paints the panel and returns its outer response. It is not an interactive terminal and does not accept input.

## ProgressRing

- `ProgressRing::new(progress: f32, radius: f32, theme)` accepts progress in the nominal `0.0..=1.0` range and a radius in egui points.
- `.show(ui) -> Response` paints a track and progress arc and returns its response.

Non-finite progress becomes zero; finite progress is clamped to `0..=1`. Invalid or too-small radius uses the minimum safe radius. The ring exposes progress semantics to egui accessibility output. It does not announce textual status, animate, or schedule repaint; pair it with text and update it from the owning workflow. `config` constants are re-exported for sizing customization/reference.
