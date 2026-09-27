# Card

Container surface and metric-card helpers for grouping native DB Pro UI content without adding interaction semantics.

## Public API

- `Card::new(theme)`: creates the themed surface wrapper.
- `Card::frame()`: returns the configured `egui::Frame` for advanced composition.
- `Card::show(ui, contents)`: renders the frame and returns the closure result.
- `card_header(ui, title, description, theme)`: renders title plus optional description.
- `card_content(ui, contents)`: wraps body content in the standard vertical card body.
- `card_footer(ui, theme, contents)`: draws a decorative separator and right-aligned footer actions.
- `MetricTrendDirection`: declares `Up`, `Down`, or `Unspecified` independently from tone.
- `MetricTrendTone`: declares `Positive`, `Negative`, `Neutral`, or `Warning` color semantics.
- `MetricTrend::new(text, is_positive)`: compatibility constructor using the legacy boolean mapping.
- `MetricCard::new(title, value, theme)`: creates a compact metric display; optional builders are `.trend(...)`, `.change(...)`, and `.icon(...)`.

## Behavior & Constraints

- Cards are non-interactive surfaces; focus and keyboard behavior belong to controls placed inside the card.
- Card colors, border, radius, and padding reuse `DbProTheme` and shared tokens.
- Footer separators are decorative and do not add accessibility semantics.
- Metric trends render direction and tone independently: `Up`/`Down` select the icon, while tone selects the semantic color. `Unspecified` renders no direction icon.
- `.change(text, is_positive)` remains supported as a compatibility shorthand: `true` maps to `Up + Positive`, and `false` maps to `Down + Negative`.
- Trend rows expose their text through egui `WidgetInfo` label metadata.
- Metric card sizing constants live in this component because they define this component's dashboard density.

## Usage Example

```rust
use db_pro_ui::components::card::{
    card_footer, card_header, Card, MetricCard, MetricTrendDirection, MetricTrendTone,
};

Card::new(theme).show(ui, |ui| {
    card_header(ui, "Connection pool", Some("Current utilization"), theme);
    ui.label("8 of 10 connections are active");
    card_footer(ui, theme, |ui| {
        ui.button("Open settings");
    });
});

MetricCard::new("Slow query rate", "0.42%", theme)
    .trend(
        "−0.15% vs last week",
        MetricTrendDirection::Down,
        MetricTrendTone::Positive,
    )
    .icon(lucide_icons::Icon::Gauge)
    .show(ui);
```

## Layers

- `mod.rs`: public entry point and stable re-exports.
- `ui.rs`: egui frame construction, layout, painting, and metric-card rendering.
- `handler.rs`: pure trend and separator decisions used by the UI layer.
- `config.rs`: Card-specific sizes that are not shared design tokens.
