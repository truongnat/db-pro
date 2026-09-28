# Feedback Components

Visual feedback indicators, loading states, keyboard badges, and labeled separators.

## Usage

```rust
// Smooth/Indeterminate Progress
Progress::new(0.65, theme).label("Import progress").show(ui);
Progress::indeterminate(theme).label("Loading schema").show(ui);

// Loading Spinner
Spinner::new(theme).size(20.0).show(ui);

// Keyboard Shortcut Badges
kbd_badge(ui, "⌘K", theme);
kbd_combo(ui, &["Ctrl", "Shift", "P"], theme);

// Labeled Separator
separator_with_text(ui, "OR CONTINUE WITH", theme);
```

## Public API

- `Progress`: Determinate/indeterminate animated progress bar. Non-finite fractions become `0`, finite fractions are clamped to `[0, 1]`; invalid or negative heights use the default (zero remains valid).
- `Spinner`: Rotating accent loading indicator with subtle track.

Both indicators expose `WidgetInfo::ProgressIndicator` accessibility semantics. Determinate progress reports a value in percent (`0..=100`); indeterminate progress and spinners report no value. Use `.label(...)` to provide a context-specific accessible label (defaults are `Progress` and `Loading`).
- `kbd_badge`: Elevated micro-container for presenting hotkeys.
- `kbd_combo`: Key sequence renderer with separating "+" symbols.
- `separator_with_text`: Horizontal rule with centered caption text.

## Architecture

- `mod.rs`: Re-exports public widgets and helpers.
- `config.rs`: Size constants, font sizes, paddings, and stroke widths.
- `handler.rs`: Pure calculations for beam widths, spinner radius, and separator lines.
- `ui.rs`: egui rendering for all feedback components.
