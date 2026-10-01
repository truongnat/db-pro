# Button

Button is the native egui control for actions in DB Pro. It supports text, icons, visual variants, size presets, loading, disabled state, tooltips, and accessible names. `ButtonGroup` places related buttons on one row with a compact gap.

## Usage

```rust
if Button::new(theme)
    .text("Save")
    .variant(ButtonVariant::Default)
    .show(ui)
    .clicked()
{
    save_changes();
}
```

Pass the current `DbProTheme` when constructing the control. Give icon-only buttons an explicit `access_label(...)`.

Read [API.md](API.md) for builders, variants, states, and usage rules. Read [DESIGN.md](DESIGN.md) when changing appearance, interaction logic, or module boundaries.
