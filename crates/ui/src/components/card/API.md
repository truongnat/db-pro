# Card API

Import the surface helpers from `db_pro_ui::components::card`.

```rust
Card::new(theme).show(ui, |ui| {
    card_header(ui, "Connection pool", Some("Current use"), theme);
    card_content(ui, |ui| ui.label("8 of 10 connections are active"));
});
```

## Container helpers

- `Card::new(theme) -> Card` creates the themed passive surface.
- `frame(&self) -> egui::Frame` returns its frame for custom composition.
- `show<R>(&self, ui, add_contents) -> R` renders the frame and returns the closure result.
- `card_header(ui, title, description, theme)` paints a title and optional description, then adds standard spacing.
- `card_content<R>(ui, add_contents) -> R` lays out vertical body content and returns its result.
- `card_footer<R>(ui, theme, add_contents) -> R` paints a decorative divider and right-aligned content, then returns the closure result.

## Metric helpers

- `MetricTrend::new(text, is_positive)` preserves the legacy mapping: `true` means Up/Positive; `false` means Down/Negative.
- `MetricTrend::with_semantics(text, direction, tone)` sets `MetricTrendDirection::{Up, Down, Unspecified}` independently from `MetricTrendTone::{Positive, Negative, Neutral, Warning}`.
- `MetricCard::new(title, value, theme)` builds a metric widget. `.change(text, is_positive)` is the compatibility shorthand; `.trend(text, direction, tone)` is the explicit form; `.icon(Icon)` adds an optional icon; `.show(ui)` paints the card.
- `MetricTrend::style(&theme)` returns the resolved `(Color32, Icon)`; `draw(&self, ui, &theme)` paints a trend label.

Card itself is not interactive. Place normal egui controls inside it for focus and keyboard behavior. Trend text communicates change but does not infer whether the underlying metric is good; choose its tone based on the metric's meaning.
